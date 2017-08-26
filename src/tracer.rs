extern crate std;
extern crate image;

use ray::{Ray};
use image::{Rgb};


// Trait for objects that can be placed in the raytracer scene
pub trait Traceable {
    // If the Ray intersects the shape, returns the distance from the Ray's
    // origin and the color at that point.
    fn intersect(&self, ray: &Ray) -> Option<(f64, image::Rgb<u8>)>;
}


// Objects that can be placed in a scene
pub struct Background {
    pub color: Rgb<u8>,
}

// The background object always intersects and returns its static color
impl Traceable for Background {
    fn intersect(&self, _: &Ray) -> Option<(f64, Rgb<u8>)> {
        Some((std::f64::INFINITY, self.color))
    }
}