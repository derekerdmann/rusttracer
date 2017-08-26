extern crate std;
extern crate image;

use ray::Ray;
use image::Rgb;


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
            let (dist, color) = bg.intersect(r).expect("Background must always intersect");
            assert_eq!(bg.color, color);
            assert_ulps_eq!(std::f64::INFINITY, dist);
        }
    }

}
