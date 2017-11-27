extern crate image;

use cgmath::{Vector3};
use image::{Pixel};

const AMBIENT_FACTOR: f64 = 0.3;
const SPECULAR_COLOR: image::Rgb<u8> = image::Rgb { data: [255, 255, 255] };


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
            specular: SPECULAR_COLOR
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
}


// Types for the color and surface of individual shapes
pub trait Material {

    // Returns the color at a particular point on the shape
    fn color(&self, point: Vector3<f64>) -> Color;

}


// Shape material that uses a single solid color at all points
pub struct SolidColorMaterial {
    color: Color,
}

impl SolidColorMaterial {
    pub fn new(color: image::Rgb<u8>) -> SolidColorMaterial {
        SolidColorMaterial { color: Color::new(color) }
    }
}

impl Material for SolidColorMaterial {
    fn color(&self, point: Vector3<f64>) -> Color {
        self.color.clone()
    }
}