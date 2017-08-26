extern crate std;
extern crate image;
extern crate cgmath;

use cgmath::{Vector3, InnerSpace, dot};
use image::{Rgb};
use tracer::{Traceable};
use ray::{Ray};

pub struct Floor {
    pub bottom_left: Vector3<f64>,
    pub top_left: Vector3<f64>,
    pub top_right: Vector3<f64>,
    pub bottom_right: Vector3<f64>,
    normal: Vector3<f64>,
    f: f64,
    pub color: Rgb<u8>,
}

impl Floor {
    pub fn new(
        bottom_left: Vector3<f64>,
        top_left: Vector3<f64>,
        top_right: Vector3<f64>,
        bottom_right: Vector3<f64>,
        color: Rgb<u8>,
    ) -> Floor {

        // Given 3 of the corners, calculate the normal and F
        let a = bottom_left - top_left;
        let b = bottom_left - bottom_right;

        let normal = Vector3 {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }.normalize();

        Floor {
            bottom_left,
            top_left,
            top_right,
            bottom_right,
            normal,
            f: -dot(normal, bottom_left),
            color,
        }
    }

    /// Translates the floor by the amount specified in the translation vector
    pub fn translate(&self, translation: Vector3<f64>) -> Floor {
        Floor::new(
            self.bottom_left + translation,
            self.top_left + translation,
            self.top_right + translation,
            self.bottom_right + translation,
            self.color,
        )
    }

    /// Rotates the floor around the X axis by the provided rotation in degrees
    pub fn rotate_x(&self, rotation: f64) -> Floor {

        /// Rotates a single vector
        fn rotate_x(v: Vector3<f64>, rotation: f64) -> Vector3<f64> {

            let theta = rotation * (std::f64::consts::PI / 180.0);

            Vector3 {
                x: v.x,
                y: v.y * theta.cos() + v.z * -theta.sin(),
                z: v.y * theta.sin() + v.z * theta.cos(),
            }
        }

        Floor::new(
            rotate_x(self.bottom_left, rotation),
            rotate_x(self.top_left, rotation),
            rotate_x(self.top_right, rotation),
            rotate_x(self.bottom_right, rotation),
            self.color,
        )
    }
}

impl Traceable for Floor {
    /// Plane intersection formula comes from CG II slides
    /// (2-2b-rt-basics-4.pdf).
    /// \omega = -(P_n . P_o + F) / (P+n . D)
    fn intersect(&self, ray: &Ray) -> Option<(f64, Rgb<u8>)> {

        let dist = -(dot(self.normal, ray.origin) + self.f) / dot(self.normal, ray.direction());

        if dist > 0.0 {

            let intersect = ray.origin + (ray.direction() * dist);
            // Make sure the value is inside the shape boundaries
            if intersect[0] >= self.bottom_left[0] && intersect[0] <= self.bottom_right[0] &&
                intersect[1] >= self.bottom_left[1] &&
                intersect[1] <= self.top_left[1]
            {

                Some((dist, self.color))
            } else {
                None
            }

        } else {
            None
        }
    }
}

