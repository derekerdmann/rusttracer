use cgmath::{Vector3, dot, InnerSpace};
use tracer::{Shape, Intersect};
use ray::Ray;
use material::Material;
use std::any::Any;

pub struct Sphere {
    pub center: Vector3<f64>,
    pub r: f64,
    pub material: Box<Material>,
}

impl PartialEq for Sphere {
    // Shapes really shouldn't be overlapping. If two different objects have the
    // same coordinates and radius but different materials, we have a bigger
    // problem.
    fn eq(&self, other: &Sphere) -> bool {
        ulps_eq!(self.center, other.center) && 
        ulps_eq!(self.r, other.r)
    }
}

impl Shape for Sphere {
    /// Sphere intersection formula comes from CG II slides
    /// (2-2b-rt-basics-4.pdf). \omega is the distance from the origin of the ray
    /// to the intersect point.
    ///
    /// \omega = (-B \pm \sqrt{B^2 - 4 * C}) / 2
    ///
    fn intersect(&self, ray: &Ray) -> Option<Intersect> {

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
            let distance = if d1 < 0.0 {
                d2
            } else if d2 < 0.0 {
                d1
            } else {
                f64::min(d1, d2)
            };

            let point = ray.extend(distance);
            let normal = (point - self.center).normalize();

            Some(Intersect {
                distance,
                point,
                normal,
                color: self.material.color(point),
                shape: self
            })
        }
    }

    fn eq(&self, other: &Shape) -> bool {
         other.as_any().downcast_ref::<Self>().map_or(false, |x| x == self)
    }

    fn as_any(&self) -> &Any { self }
}


#[cfg(test)]
mod tests {

    extern crate image;

    use cgmath::vec3;
    use tracer::Shape;
    use sphere::Sphere;
    use ray::Ray;
    use material::SolidColorMaterial;

    // Tests collisions with a sphere, pointing at center
    #[test]
    fn intersect() {

        let color = image::Rgb([255, 255, 0]);

        let sphere = Sphere {
            center: vec3(0.0, 0.0, 1.0),
            r: 0.5,
            material: Box::new(SolidColorMaterial::new(color)),
        };

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));
        let intersect = sphere.intersect(&r).expect(
            "Ray should intersect with sphere",
        );
        assert_eq!(color, intersect.color.diffuse());
        assert_ulps_eq!(0.5, intersect.distance);
    }


    #[test]
    fn intersect_tangent() {

        let color = image::Rgb([255, 255, 0]);

        let sphere = Sphere {
            center: vec3(0.0, 0.0, 1.0),
            r: 0.5,
            material: Box::new(SolidColorMaterial::new(color)),
        };

        let r = Ray::new(vec3(0.0, 0.5, 0.0), vec3(0.0, 0.0, 1.0));
        let intersect = sphere.intersect(&r).expect(
            "Ray should intersect with sphere at tangent",
        );

        assert_eq!(color, intersect.color.diffuse());
        assert_ulps_eq!(1.0, intersect.distance);
    }
}
