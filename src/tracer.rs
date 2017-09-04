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
pub trait Traceable {
    // If the Ray intersects the shape, returns the distance from the Ray's
    // origin and the color at that point.
    fn intersect(&self, ray: &Ray) -> Option<Intersect>;
}


// Objects that can be placed in a scene
pub struct Background {
    pub color: image::Rgb<u8>,
}

// The background object always intersects and returns its static color
impl Traceable for Background {
    fn intersect(&self, _: &Ray) -> Option<Intersect> {
        Some(Intersect {
            distance: std::f64::INFINITY,
            point: None,
            color: self.color,
        })
    }
}

// The main tracer function. Fires the ray into the scene, calculating the
// objects it intersects and the final output color
pub fn trace(r: Ray, shapes: &Vec<&Traceable>, background: &Traceable) -> image::Rgb<u8> {

    let bg = background.intersect(&r).expect(
        "Background must always intersect!",
    );

    let intersect = shapes.iter().fold(bg, |best, &shape| {
        if let Some(inter) = shape.intersect(&r) {
            if inter.distance < best.distance {
                inter
            } else {
                best
            }
        } else {
            best
        }
    });

    intersect.color
}


#[cfg(test)]
mod tests {

    use std;
    use image::Rgb;
    use cgmath::vec3;
    use ray::Ray;
    use tracer::{Traceable, Background};

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

}
