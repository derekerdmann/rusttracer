extern crate image;

use cgmath::{dot, InnerSpace, Vector3, vec3};
use tracer::{Intersect, Shape};
use ray::Ray;
use tracer::shape_intersect;
use material::{modulate, modulate_scalar, AMBIENT_FACTOR};
use image::Pixel;
use material::{blend_add, blend_mult};

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

    let ambient = modulate_scalar(intersect.color.ambient(), AMBIENT_FACTOR);

    lights.iter().fold(ambient, |result, &light| {
        // Shadow ray
        let s = (light.position - intersect.point).normalize();

        match shape_intersect(&Ray::new(intersect.point, s), shapes, Some(intersect.shape)) {
            None => {
                let diffuse_dot = dot(s, n);
                if diffuse_dot > 0.0 {
                    blend_add(
                        result,
                        modulate_scalar(
                            blend_mult(intersect.color.diffuse(), light.color),
                            diffuse_dot,
                        ),
                    )
                } else {
                    result
                }
            }
            Some(blocking) => result,
        }
    })
}


#[cfg(test)]
mod tests {

    #[test]
    fn thing() {}
}
