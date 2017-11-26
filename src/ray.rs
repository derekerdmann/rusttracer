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

    // Extends the ray by the distance specified. Resulting vector will not be
    // normalized.
    pub fn extend(&self, distance: f64) -> Vector3<f64> {
        self.origin + (self.direction * distance)
    }
}


#[cfg(test)]
mod tests {

    use cgmath::{InnerSpace, vec3};

    use ray::Ray;

    // Tests that direction vectors of any magnitude get normalized and have a
    // magnitude of 1.0
    #[test]
    fn auto_normalize() {
        let r1 = Ray::new(vec3(0.0, 0.0, 0.0), vec3(2.0, 0.0, 0.0));
        assert_ulps_eq!(r1.direction().magnitude(), 1.0);

        let r2 = Ray::new(vec3(0.0, 0.0, 0.0), vec3(2.889, 90.0, -30.5));
        assert_ulps_eq!(r2.direction().magnitude(), 1.0);
    }

    // Tests that an extended ray has the correct magnitude and points in the
    // same direction as the original ray
    #[test]
    fn extend() {
        let r1 = Ray::new(vec3(0.0, 0.0, 0.0), vec3(2.0, 0.0, 0.0));
        assert_ulps_eq!(r1.direction().magnitude(), 1.0);

        let extended1 = r1.extend(2.0);
        assert_ulps_eq!(extended1.magnitude(), 2.0);
        assert_ulps_eq!(r1.origin + r1.direction(), extended1.normalize());
    }

}
