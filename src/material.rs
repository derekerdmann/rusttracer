extern crate image;

use cgmath::Vector3;
use image::Pixel;
use std::cmp;
use std::u8;

pub const AMBIENT_FACTOR: f64 = 0.3;
const SPECULAR_COLOR: image::Rgb<u8> = image::Rgb {
    data: [255, 255, 255],
};
const SHININESS: f64 = 16.0;


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


// Types for the color and surface of individual shapes
pub trait Material {
    // Returns the color at a particular point on the shape
    fn color(&self, point: Vector3<f64>) -> Color;
}


// Shape material that uses a single solid color at all points
#[derive(PartialEq, Eq, Debug)]
pub struct SolidColorMaterial {
    color: Color,
}

impl SolidColorMaterial {
    pub fn new(color: image::Rgb<u8>) -> SolidColorMaterial {
        SolidColorMaterial {
            color: Color::new(color),
        }
    }
}

impl Material for SolidColorMaterial {
    fn color(&self, _: Vector3<f64>) -> Color {
        self.color.clone()
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
pub fn blend_add(color1: image::Rgb<u8>, color2: image::Rgb<u8>) -> image::Rgb<u8>{
    color1.map2(&color2, |c1, c2| {
        cmp::min((c1 as u16) + (c2 as u16), u8::MAX as u16) as u8
    })
}

// Blends two colors by multiplying each channel
pub fn blend_mult(color1: image::Rgb<u8>, color2: image::Rgb<u8>) -> image::Rgb<u8>{
    color1.map2(&color2, |c1, c2| {
        ((c1 as u16) * (c2 as u16) / u8::MAX as u16) as u8
    })
}
