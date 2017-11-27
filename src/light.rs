extern crate image;

use cgmath::{dot, InnerSpace, Vector3, vec3};
use tracer::{Intersect, Shape};
use ray::Ray;
use tracer::shape_intersect;
use material::{modulate, modulate_scalar, AMBIENT_FACTOR};
use image::Pixel;

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
    let k_a = intersect.color.ambient();
    let k_d = intersect.color.diffuse();
    let k_s = intersect.color.specular();
    let alpha = intersect.color.shininess();
    let n = intersect.normal;

    let ambient = k_a; //modulate(k_a, vec3(1.0, 1.0, 1.0 * AMBIENT_FACTOR));

    lights.iter().fold(ambient, |result, &light| {
        let l = (light.position - intersect.point).normalize();

        let block = shape_intersect(&Ray::new(intersect.point, l), shapes, Some(intersect.shape));

        match block {
            None => {
                intersect.color.diffuse()

                //let mut diffuse = light.color.clone();
                //diffuse.blend(&k_d);

                //let diffuse_dot = dot(l, n);
                //if diffuse_dot > 0.0 {
                //    modulate_scalar(diffuse, diffuse_dot);

                //    let mut blended = result.clone();
                //    blended.blend(&light.color);
                //    blended
                //} else {
                //    result
                //}
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
