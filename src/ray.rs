extern crate cgmath;

use cgmath::{Vector3, InnerSpace};

// Individual ray that is fired through the scene
// Direction is private because it must always be normalized
pub struct Ray {
    pub origin: Vector3<f64>,
    direction: Vector3<f64>,
}

impl Ray {

    // Constructs a Ray that starts at origin and points at direction. Direction
    // is normalized automatically.
    pub fn new(origin: Vector3<f64>, direction: Vector3<f64>) -> Ray {
        Ray {
            origin: origin,
            direction: direction.normalize(),
        }
    }

    // Accessor for direction vector. This is normalized when the ray is
    // constructed, guaranteeing that the vector magnitude is always 1.0
    pub fn direction(&self) -> Vector3<f64> {
        self.direction 
    }
}