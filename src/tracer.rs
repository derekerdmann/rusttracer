extern crate image;
extern crate std;

use cgmath::{InnerSpace, Vector3};
use ray::Ray;
use light::{phong, Color, Light};
use std::any::Any;

// Represents the intersection of a Ray with an object
pub struct Intersect<'a> {
    // Distance from the origin where the intersect occurs
    pub distance: f64,

    // Point in space where the intersect occurs
    pub point: Vector3<f64>,

    // Normal vector from the surface of the shape at this intersect
    pub normal: Vector3<f64>,

    // Color of the object where the intersect occurs
    pub color: &'a Color,

    // Shape that the ray intersects
    pub shape: &'a Shape,
}

// Trait for objects that can be placed in the raytracer scene
pub trait Shape {
    // If the Ray intersects the shape, returns the distance from the Ray's
    // origin and the color at that point.
    fn intersect(&self, ray: &Ray) -> Option<Intersect>;

    // Used to downcast and check equality
    fn eq(&self, other: &Shape) -> bool;
    fn as_any(&self) -> &Any;
}

impl<'a, 'b> PartialEq<Shape + 'b> for Shape + 'a {
    fn eq(&self, other: &(Shape + 'b)) -> bool {
        Shape::eq(self, other)
    }
}


// Objects that can be placed in a scene
pub struct Background {
    pub color: image::Rgb<u8>,
}

// Of all shapes that intersect with this ray, select the closest one that's in
// front of the starting point.
pub fn shape_intersect<'a>(
    r: &Ray,
    shapes: &Vec<&'a Shape>,
    exclude: Option<&Shape>,
) -> Option<Intersect<'a>> {
    shapes
        .iter()
        .filter(|&shape| exclude.map_or(true, |e| &e != shape))
        .filter_map(|&shape| shape.intersect(&r))
        .filter(|intersect| intersect.distance >= 0.0)
        .min_by(|first, second| {
            first.distance.partial_cmp(&second.distance).unwrap()
        })
}

// The main tracer function. Fires the ray into the scene, calculating the
// objects it intersects and the final output color
pub fn trace(
    r: Ray,
    shapes: &Vec<&Shape>,
    lights: &Vec<&Light>,
    background: &Background,
) -> image::Rgb<u8> {
    match shape_intersect(&r, shapes, None) {
        Some(intersect) => phong(
            &intersect,
            shapes,
            lights,
            (r.direction() - r.origin).normalize(),
        ),
        None => background.color,
    }
}


#[cfg(test)]
mod tests {

    extern crate image;

    use cgmath::vec3;
    use ray::Ray;
    use tracer::Shape;
    use floor::Floor;
    use light::Color;
    use super::shape_intersect;

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
            Color::new(color1),
            Color::new(color1),
        );

        let f2 = Floor::new(
            vec3(-1.0, -1.0, 2.0),
            vec3(-1.0, 1.0, 2.0),
            vec3(1.0, -1.0, 2.0),
            vec3(1.0, 1.0, 2.0),
            Color::new(color2),
            Color::new(color2),
        );

        let shapes: Vec<&Shape> = vec![&f1, &f2];

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));

        let intersect =
            shape_intersect(&r, &shapes, None).expect("Both of these objects should intersect");

        assert_ulps_eq!(1.0, intersect.distance);
    }

    // Tests that a shape is excluded if specified
    #[test]
    fn intersect_exclude() {
        let color1 = image::Rgb([255, 0, 0]);
        let color2 = image::Rgb([0, 255, 0]);

        let f1 = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Color::new(color1),
            Color::new(color1),
        );

        let f2 = Floor::new(
            vec3(-1.0, -1.0, 2.0),
            vec3(-1.0, 1.0, 2.0),
            vec3(1.0, -1.0, 2.0),
            vec3(1.0, 1.0, 2.0),
            Color::new(color2),
            Color::new(color2),
        );

        let shapes: Vec<&Shape> = vec![&f1, &f2];

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));

        // Exclude a shape that already isn't closest
        let intersect = shape_intersect(&r, &shapes, Some(&f2)).expect("f1 should intersect");
        assert_ulps_eq!(1.0, intersect.distance);

        // Exclude the closest shape
        let intersect = shape_intersect(&r, &shapes, Some(&f1)).expect("f2 should intersect");
        assert_ulps_eq!(2.0, intersect.distance);

        // Exclude the only shape
        let shapes: Vec<&Shape> = vec![&f1];
        let intersect = shape_intersect(&r, &shapes, Some(&f1));
        assert!(intersect.is_none());

    }

}
