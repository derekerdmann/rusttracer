extern crate image;
extern crate cgmath;

use cgmath::{Vector3, dot};
use image::Rgb;
use tracer::Traceable;
use ray::Ray;

pub struct Sphere {
    pub center: Vector3<f64>,
    pub r: f64,
    pub color: Rgb<u8>,
}

impl Traceable for Sphere {
    /// Sphere intersection formula comes from CG II slides
    /// (2-2b-rt-basics-4.pdf). \omega is the distance from the origin of the ray
    /// to the intersect point.
    ///
    /// \omega = (-B \pm \sqrt{B^2 - 4 * C}) / 2
    ///
    fn intersect(&self, ray: &Ray) -> Option<(f64, Rgb<u8>)> {

        // B=2 * (dx(x_o −x_c)+dy(y_o −y_c)+dz(z_o −z_c))
        // which is just the dot product
        // B = 2 * (d . (origin - center))
        let b = 2.0 * dot(ray.direction(), ray.origin - self.center);

        // C = (x_o −x_c)^2 +(y_o −y_c)^2 +(z_o −z_c)^2 − r^2
        // which also uses the dot product:
        // tmp = origin - center;
        // C = tmp . tmp - r^2
        let c_sub = ray.origin - self.center;
        let c = dot(c_sub, c_sub) - (self.r * self.r);

        // Partial quadratic solution
        let partial = b * b - 4.0 * c;

        if partial < 0.0 {
            None

        } else {
            let d1 = (-b + partial) / 2.0;
            let d2 = (-b - partial) / 2.0;

            // There are two solutions, so return the smallest positive result.
            // The larger value would be the far side of the sphere.
            let d = if d1 < 0.0 {
                d2
            } else if d2 < 0.0 {
                d1
            } else {
                f64::min(d1, d2)
            };

            Some((d, self.color))
        }
    }
}
