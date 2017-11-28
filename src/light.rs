extern crate image;

use cgmath::{dot, InnerSpace, Vector3};
use tracer::{Intersect, Shape};
use ray::Ray;
use tracer::shape_intersect;
use std::{cmp, u8};
use image::Pixel;

pub const AMBIENT_FACTOR: f64 = 0.3;
const SPECULAR_COLOR: image::Rgb<u8> = image::Rgb {
    data: [255, 255, 255],
};
const SHININESS: f64 = 16.0;

// Represents a single point light that's placed within the scene
pub struct Light {
    pub position: Vector3<f64>,
    pub color: image::Rgb<u8>,
}

// Color of a shape at a specific point. Includes the components needed for
// phong shading, automatically derived from the primary color.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Color {
    ambient: image::Rgb<u8>,
    diffuse: image::Rgb<u8>,
    specular: image::Rgb<u8>,
}

impl Color {
    pub fn new(color: image::Rgb<u8>) -> Color {
        Color {
            ambient: color.map(|channel| ((channel as f64) * AMBIENT_FACTOR) as u8),
            diffuse: color,
            specular: SPECULAR_COLOR,
        }
    }

    pub fn ambient(&self) -> image::Rgb<u8> {
        self.ambient
    }

    pub fn diffuse(&self) -> image::Rgb<u8> {
        self.diffuse
    }

    pub fn specular(&self) -> image::Rgb<u8> {
        self.specular
    }

    pub fn shininess(&self) -> f64 {
        SHININESS
    }
}


// Modulates a color by the specified vector
pub fn modulate_scalar(color: image::Rgb<u8>, amount: f64) -> image::Rgb<u8> {
    image::Rgb([
        (color[0] as f64 * amount) as u8,
        (color[1] as f64 * amount) as u8,
        (color[2] as f64 * amount) as u8,
    ])
}

// Blends two colors by multiplying each channel
pub fn blend_add(color1: image::Rgb<u8>, color2: image::Rgb<u8>) -> image::Rgb<u8> {
    color1.map2(&color2, |c1, c2| {
        cmp::min((c1 as u16) + (c2 as u16), u8::MAX as u16) as u8
    })
}

// Blends two colors by multiplying each channel
pub fn blend_mult(color1: image::Rgb<u8>, color2: image::Rgb<u8>) -> image::Rgb<u8> {
    color1.map2(&color2, |c1, c2| {
        ((c1 as u16) * (c2 as u16) / u8::MAX as u16) as u8
    })
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
        let r = (s - 2.0 * (dot(s, n) / n.magnitude().powi(2)) * n).normalize();

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
