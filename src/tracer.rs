extern crate std;
extern crate image;

use cgmath::Vector3;
use ray::Ray;
use material::Color;

// Represents the intersection of a Ray with an object
pub struct Intersect {
    // Distance from the origin where the intersect occurs
    pub distance: f64,

    // Point in space where the intersect occurs
    pub point: Vector3<f64>,

    // Normal vector from the surface of the shape at this intersect
    pub normal: Vector3<f64>,

    // Color of the object where the intersect occurs
    pub color: Color,
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
        Some(intersect) => intersect.color.diffuse(),
        None => background.color,
    }
}


#[cfg(test)]
mod tests {

    extern crate image;

    use std::rc::Rc;
    use cgmath::vec3;
    use ray::Ray;
    use tracer::{Shape};
    use floor::Floor;
    use super::shape_intersect;
    use material::SolidColorMaterial;

    // Tests that the closest shape is selected
    #[test]
    fn intersect_ordering() {

        let color1 = image::Rgb([255, 0, 0]);
        let color2 = image::Rgb([0, 255, 0]);

        let f1 = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Rc::new(SolidColorMaterial::new(color1)),
        );

        let f2 = Floor::new(
            vec3(-1.0, -1.0, 2.0),
            vec3(-1.0, 1.0, 2.0),
            vec3(1.0, -1.0, 2.0),
            vec3(1.0, 1.0, 2.0),
            Rc::new(SolidColorMaterial::new(color2)),
        );

        let shapes: Vec<&Shape> = vec![&f1, &f2];

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));

        let intersect = shape_intersect(&r, &shapes).expect("Both of these objects should intersect");

        assert_ulps_eq!(1.0, intersect.distance);
        assert_eq!(color1, intersect.color.diffuse());
    }

}
