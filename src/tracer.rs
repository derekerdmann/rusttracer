extern crate std;
extern crate image;

use cgmath::Vector3;
use ray::Ray;

// Represents the intersection of a Ray with an object
pub struct Intersect {
    // Distance from the origin where the intersect occurs
    pub distance: f64,

    // Point in space where the intersect occurs
    pub point: Option<Vector3<f64>>,

    // Color of the object where the intersect occurs
    pub color: image::Rgb<u8>,
}

// Trait for objects that can be placed in the raytracer scene
pub trait Shape {
    // If the Ray intersects the shape, returns the distance from the Ray's
    // origin and the color at that point.
    fn intersect(&self, ray: &Ray) -> Option<Intersect>;
}


// Objects that can be placed in a scene
pub struct Background {
    pub color: image::Rgb<u8>,
}

// The background object always intersects and returns its static color
impl Shape for Background {
    fn intersect(&self, _: &Ray) -> Option<Intersect> {
        Some(Intersect {
            distance: std::f64::INFINITY,
            point: None,
            color: self.color,
        })
    }
}

// Of all shapes that intersect with this ray, select the closest one.
fn shape_intersect(r: &Ray, shapes: &Vec<&Shape>) -> Option<Intersect> {
    shapes
        .iter()
        .filter_map(|&shape| shape.intersect(&r))
        .min_by(|first, second| {
            first.distance.partial_cmp(&second.distance).unwrap()
        })
}

// The main tracer function. Fires the ray into the scene, calculating the
// objects it intersects and the final output color
pub fn trace(r: Ray, shapes: &Vec<&Shape>, background: &Background) -> image::Rgb<u8> {
    match shape_intersect(&r, shapes) {
        Some(intersect) => intersect.color,
        None => background.color,
    }
}


#[cfg(test)]
mod tests {

    use std;
    use image::Rgb;
    use cgmath::vec3;
    use ray::Ray;
    use tracer::{Traceable, Background};
    use floor::Floor;
    use super::shape_intersect;

    // Tests that the background always intersects with any Ray
    #[test]
    fn background_always_intersects() {
        let rays = [
            Ray::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0)),
            Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0)),
            Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0)),
            Ray::new(vec3(0.0, 0.0, 0.0), vec3(-1.0, 0.0, 0.0)),
            Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, -1.0, 0.0)),
            Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, -1.0)),
        ];

        let bg = Background { color: Rgb([0, 175, 215]) };
        for r in rays.iter() {
            let intersect = bg.intersect(r).expect("Background must always intersect");
            assert_eq!(bg.color, intersect.color);
            assert_ulps_eq!(std::f64::INFINITY, intersect.distance);
        }
    }

    // Tests that the closest shape is selected
    #[test]
    fn intersect_ordering() {

        let f1 = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Rgb([255, 0, 0]),
        );

        let f2 = Floor::new(
            vec3(-1.0, -1.0, 2.0),
            vec3(-1.0, 1.0, 2.0),
            vec3(1.0, -1.0, 2.0),
            vec3(1.0, 1.0, 2.0),
            Rgb([0, 255, 0]),
        );

        let shapes: Vec<&Traceable> = vec![&f1, &f2];

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));

        let intersect = shape_intersect(&r, &shapes).expect("Both of these objects should intersect");

        assert_ulps_eq!(1.0, intersect.distance);
        assert_eq!(f1.color, intersect.color);
    }

}
