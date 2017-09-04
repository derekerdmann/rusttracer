use cgmath::{Vector3, dot};
use tracer::{Traceable, Intersect};
use ray::Ray;

// Represents a single point light that's placed within the scene
struct Light {
    pub position: Vector3<f64>,
    pub color: image::Rgb<u8>,
}


#[cfg(test)]
mod tests {

    #[test]
    fn thing() {
    }
}