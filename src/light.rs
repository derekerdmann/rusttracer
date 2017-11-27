extern crate image;

use cgmath::{dot, InnerSpace, Vector3};
use tracer::{Intersect, Shape};
use ray::Ray;
use tracer::shape_intersect;
use material::{modulate_scalar, AMBIENT_FACTOR, blend_add, blend_mult};

// Represents a single point light that's placed within the scene
pub struct Light {
    pub position: Vector3<f64>,
    pub color: image::Rgb<u8>,
}


// Performs phong shading in a scene
pub fn phong(
    intersect: &Intersect,
    shapes: &Vec<&Shape>,
    lights: &Vec<&Light>,
    v: Vector3<f64>,
) -> image::Rgb<u8> {
    let n = intersect.normal;

    // Start with the base ambient lighting
    let ambient = modulate_scalar(intersect.color.ambient(), AMBIENT_FACTOR);

    lights.iter().fold(ambient, |result, &light| {
        // Shadow ray
        let s = (light.position - intersect.point).normalize();

        // Reflected vector
        let r = s - 2.0 * (dot(s, n) / n.magnitude().powi(2)) * n;

        match shape_intersect(&Ray::new(intersect.point, s), shapes, Some(intersect.shape)) {
            None => {

                // Calculate diffuse light component
                let diffuse_dot = dot(s, n);
                let result = if diffuse_dot > 0.0 {
                    blend_add(
                        result,
                        modulate_scalar(
                            blend_mult(intersect.color.diffuse(), light.color),
                            diffuse_dot,
                        ),
                    )
                } else {
                    result
                };

                // Calculate the specular component
                let specular_dot = dot(r, v);
                let result = if specular_dot > 0.0 {
                    blend_add(
                        result,
                        modulate_scalar(
                            blend_mult(intersect.color.specular(), light.color),
                            specular_dot.powf(intersect.color.shininess()),
                        ),
                    )
                } else {
                    result
                };

                result
            }
            Some(_) => result,
        }
    })
}


#[cfg(test)]
mod tests {

    #[test]
    fn thing() {}
}
