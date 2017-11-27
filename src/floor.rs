extern crate std;

use cgmath::{dot, InnerSpace, Vector3};
use tracer::{Intersect, Shape};
use ray::Ray;
use material::Material;
use std::rc::Rc;
use std::any::Any;

pub struct Floor {
    pub bottom_left: Vector3<f64>,
    pub top_left: Vector3<f64>,
    pub top_right: Vector3<f64>,
    pub bottom_right: Vector3<f64>,
    normal: Vector3<f64>,
    f: f64,
    pub material: Rc<Material>,
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

impl Floor {
    pub fn new(
        bottom_left: Vector3<f64>,
        top_left: Vector3<f64>,
        top_right: Vector3<f64>,
        bottom_right: Vector3<f64>,
        material: Rc<Material>,
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
            material,
        }
    }

    /// Translates the floor by the amount specified in the translation vector
    pub fn translate(&self, translation: Vector3<f64>) -> Floor {
        Floor::new(
            self.bottom_left + translation,
            self.top_left + translation,
            self.top_right + translation,
            self.bottom_right + translation,
            Rc::clone(&self.material),
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
            Rc::clone(&self.material),
        )
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
                    color: self.material.color(intersect),
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

    extern crate image;

    use std::rc::Rc;
    use cgmath::vec3;
    use tracer::Shape;
    use floor::Floor;
    use ray::Ray;
    use material::SolidColorMaterial;

    // Tests collisions with a simple floor that hasn't been rotated or
    // translated
    #[test]
    fn intersect() {
        let color = image::Rgb([255, 0, 0]);

        let floor = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Rc::new(SolidColorMaterial::new(color)),
        );

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
        let intersect = floor
            .intersect(&r)
            .expect("Ray should intersect with floor");
        assert_eq!(color, intersect.color.diffuse());
        assert_ulps_eq!(1.0, intersect.distance);

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(1.0, -1.0, 1.0));
        let intersect = floor
            .intersect(&r)
            .expect("Ray should intersect with floor at edge");
        assert_eq!(color, intersect.color.diffuse());

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0));
        let result = floor.intersect(&r);
        assert!(result.is_none());
    }

    // Tests collisions after translating the floor
    #[test]
    fn intersect_translated() {
        let color = image::Rgb([255, 0, 0]);

        let floor = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Rc::new(SolidColorMaterial::new(color)),
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
        assert_eq!(color, intersect.color.diffuse());
        assert_ulps_eq!(1.0, intersect.distance);

        // Ray from origin, toward new center
        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 2.0, 3.0));
        let intersect = floor
            .intersect(&r)
            .expect("Ray should intersect with floor");
        assert_eq!(color, intersect.color.diffuse());
    }

    // Tests collisions after rotating the floor
    #[test]
    fn intersect_rotate_x() {
        let color = image::Rgb([255, 0, 0]);

        let floor = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Rc::new(SolidColorMaterial::new(color)),
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
        assert_eq!(color, intersect.color.diffuse());
        assert_ulps_eq!(1.0, intersect.distance);
    }
}
