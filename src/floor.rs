extern crate std;

use cgmath::{dot, Angle, InnerSpace, Vector3};
use tracer::{Intersect, Shape};
use ray::Ray;
use std::any::Any;
use light::Material;

const SUBDIVISIONS_X: f64 = 7.0;
const SUBDIVISIONS_Y: f64 = 7.0;

pub struct Floor {
    pub bottom_left: Vector3<f64>,
    pub top_left: Vector3<f64>,
    pub top_right: Vector3<f64>,
    pub bottom_right: Vector3<f64>,
    normal: Vector3<f64>,
    f: f64,
    width: f64,
    height: f64,
    pub color1: Material,
    pub color2: Material,
}

impl Floor {
    pub fn new(
        bottom_left: Vector3<f64>,
        top_left: Vector3<f64>,
        top_right: Vector3<f64>,
        bottom_right: Vector3<f64>,
        color1: Material,
        color2: Material,
    ) -> Floor {
        // Given 3 of the corners, calculate the normal and F
        let a = bottom_left - top_left;
        let b = bottom_left - bottom_right;

        let normal = Vector3 {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }.normalize();

        let width = (bottom_left - bottom_right).magnitude();
        let height = (bottom_left - top_left).magnitude();

        Floor {
            bottom_left,
            top_left,
            top_right,
            bottom_right,
            normal,
            f: -dot(normal, bottom_left),
            width,
            height,
            color1,
            color2,
        }
    }

    /// Translates the floor by the amount specified in the translation vector
    pub fn translate(&self, translation: Vector3<f64>) -> Floor {
        Floor::new(
            self.bottom_left + translation,
            self.top_left + translation,
            self.top_right + translation,
            self.bottom_right + translation,
            self.color1.clone(),
            self.color2.clone(),
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
            self.color1.clone(),
            self.color2.clone(),
        )
    }

    // Calculates the correct color at a specific intersect
    fn color_at(&self, intersect: &Vector3<f64>) -> &Material {
        let hyp = intersect - self.bottom_left;
        let hyp_length = hyp.magnitude();
        let angle = (self.bottom_right - self.bottom_left).angle(hyp);

        let x = angle.cos() * hyp_length;
        let y = angle.sin() * hyp_length;

        let x_parity = (x / (self.width / SUBDIVISIONS_X)) as u64 % 2;
        let y_parity = (y / (self.height / SUBDIVISIONS_Y)) as u64 % 2;

        if x_parity == y_parity {
            &self.color1
        } else {
            &self.color2
        }
    }
}

impl PartialEq for Floor {
    // Shapes really shouldn't be overlapping. If two different objects have the
    // same coordinates and dimensions but different materials, we have a bigger
    // problem.
    fn eq(&self, other: &Floor) -> bool {
        ulps_eq!(self.bottom_left, other.bottom_left) && ulps_eq!(self.top_left, other.top_left)
            && ulps_eq!(self.top_right, other.top_right)
            && ulps_eq!(self.bottom_right, other.bottom_right)
            && ulps_eq!(self.normal, other.normal) && ulps_eq!(self.f, other.f)
    }
}


impl Shape for Floor {
    /// Plane intersection formula comes from CG II slides
    /// (2-2b-rt-basics-4.pdf).
    /// \omega = -(P_n . P_o + F) / (P+n . D)
    fn intersect(&self, ray: &Ray) -> Option<Intersect> {
        let distance = -(dot(self.normal, ray.origin) + self.f) / dot(self.normal, ray.direction());

        if distance > 0.0 {
            let intersect = ray.extend(distance);
            // Make sure the value is inside the shape boundaries
            if intersect.x >= self.bottom_left.x && intersect.x <= self.bottom_right.x
                && intersect.y >= self.bottom_left.y
                && intersect.y <= self.top_left.y
            {
                Some(Intersect {
                    distance,
                    point: intersect,
                    normal: self.normal,
                    color: self.color_at(&intersect),
                    shape: self,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn eq(&self, other: &Shape) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |x| x == self)
    }

    fn as_any(&self) -> &Any {
        self
    }
}


#[cfg(test)]
mod tests {

    use cgmath::vec3;
    use tracer::Shape;
    use floor::Floor;
    use ray::Ray;
    use light::{Material, Rgb};

    // Tests collisions with a simple floor that hasn't been rotated or
    // translated
    #[test]
    fn intersect() {
        let color = Rgb::new([255, 0, 0]);

        let floor = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Material::new(color.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
            Material::new(color.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
        );

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
        let intersect = floor
            .intersect(&r)
            .expect("Ray should intersect with floor");
        assert_eq!(&color, intersect.color.diffuse());
        assert_ulps_eq!(1.0, intersect.distance);

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(1.0, -1.0, 1.0));
        let intersect = floor
            .intersect(&r)
            .expect("Ray should intersect with floor at edge");
        assert_eq!(&color, intersect.color.diffuse());

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0));
        let result = floor.intersect(&r);
        assert!(result.is_none());
    }

    // Tests collisions after translating the floor
    #[test]
    fn intersect_translated() {
        let color = Rgb::new([255, 0, 0]);

        let floor = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Material::new(color.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
            Material::new(color.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
        );

        let floor = floor.translate(vec3(1.0, 2.0, 3.0));

        // Ray from origin, down Z axis
        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
        let result = floor.intersect(&r);
        assert!(result.is_none());

        // Ray toward floor, from new center
        let r = Ray::new(vec3(1.0, 2.0, 3.0), vec3(0.0, 0.0, 1.0));
        let intersect = floor
            .intersect(&r)
            .expect("Ray should intersect with floor");
        assert_eq!(&color, intersect.color.diffuse());
        assert_ulps_eq!(1.0, intersect.distance);

        // Ray from origin, toward new center
        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 2.0, 3.0));
        let intersect = floor
            .intersect(&r)
            .expect("Ray should intersect with floor");
        assert_eq!(&color, intersect.color.diffuse());
    }

    // Tests collisions after rotating the floor
    #[test]
    fn intersect_rotate_x() {
        let color = Rgb::new([255, 0, 0]);

        let floor = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Material::new(color.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
            Material::new(color.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
        );

        let floor = floor.rotate_x(-90.0);

        // Ray from origin, down Z axis
        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
        let result = floor.intersect(&r);
        assert!(result.is_none());

        // Ray from origin, toward Y axis
        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
        let intersect = floor
            .intersect(&r)
            .expect("Ray should intersect with floor");
        assert_eq!(&color, intersect.color.diffuse());
        assert_ulps_eq!(1.0, intersect.distance);
    }
}
