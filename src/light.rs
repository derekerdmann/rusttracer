extern crate image;

use cgmath::{dot, InnerSpace, Vector3};
use tracer::{Intersect, Shape};
use ray::Ray;
use tracer::{shape_intersect, transmission_ray};
use std::{cmp, u8};
use image::Pixel;
use std::ops::{Add, Mul};

pub const AMBIENT_FACTOR: f64 = 0.3;
const SPECULAR_COLOR: Rgb = Rgb {
    color: image::Rgb {
        data: [255, 255, 255],
    },
};
const SHININESS: f64 = 20.0;
const MAX_SHADOW_DEPTH: u8 = 4;


// Wrapper for image::Rgb that has overloaded operators
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rgb {
    pub color: image::Rgb<u8>,
}

impl Rgb {
    pub fn new(data: [u8; 3]) -> Rgb {
        Rgb {
            color: image::Rgb(data),
        }
    }
}


impl<'a, 'b> Mul<&'b Rgb> for &'a Rgb {
    type Output = Rgb;
    fn mul(self, rhs: &'b Rgb) -> Rgb {
        Rgb {
            color: self.color.map2(&rhs.color, |c1, c2| {
                ((c1 as u16) * (c2 as u16) / u8::MAX as u16) as u8
            }),
        }
    }
}

impl<'a> Mul<&'a Rgb> for Rgb {
    type Output = Rgb;
    fn mul(self, rhs: &'a Rgb) -> Rgb {
        &self * rhs
    }
}

impl<'a> Mul<Rgb> for &'a Rgb {
    type Output = Rgb;
    fn mul(self, rhs: Rgb) -> Rgb {
        self * &rhs
    }
}

impl Mul<Rgb> for Rgb {
    type Output = Rgb;
    fn mul(self, rhs: Rgb) -> Rgb {
        &self * &rhs
    }
}


impl<'a, 'b> Add<&'b Rgb> for &'a Rgb {
    type Output = Rgb;
    fn add(self, rhs: &'b Rgb) -> Rgb {
        Rgb {
            color: self.color.map2(&rhs.color, |c1, c2| {
                cmp::min((c1 as u16) + (c2 as u16), u8::MAX as u16) as u8
            }),
        }
    }
}

impl<'a> Add<&'a Rgb> for Rgb {
    type Output = Rgb;
    fn add(self, rhs: &'a Rgb) -> Rgb {
        &self + rhs
    }
}

impl<'a> Add<Rgb> for &'a Rgb {
    type Output = Rgb;
    fn add(self, rhs: Rgb) -> Rgb {
        self + &rhs
    }
}

impl Add<Rgb> for Rgb {
    type Output = Rgb;
    fn add(self, rhs: Rgb) -> Rgb {
        &self + &rhs
    }
}


impl<'a> Mul<f64> for &'a Rgb {
    type Output = Rgb;
    fn mul(self, rhs: f64) -> Rgb {
        Rgb {
            color: self.color
                .map(|c| cmp::min((c as f64 * rhs) as u16, u8::MAX as u16) as u8),
        }
    }
}

impl<'a> Mul<u8> for &'a Rgb {
    type Output = Rgb;
    fn mul(self, rhs: u8) -> Rgb {
        self * rhs as f64
    }
}

impl Mul<f64> for Rgb {
    type Output = Rgb;
    fn mul(self, rhs: f64) -> Rgb {
        &self * rhs
    }
}

impl Mul<u8> for Rgb {
    type Output = Rgb;
    fn mul(self, rhs: u8) -> Rgb {
        &self * rhs
    }
}


// Represents a single point light that's placed within the scene
pub struct Light {
    pub position: Vector3<f64>,
    pub color: Rgb,
}

// Color of a shape at a specific point. Includes the components needed for
// phong shading, automatically derived from the primary color.
#[derive(Clone, PartialEq, Debug)]
pub struct Material {
    // Phong colors
    ambient: Rgb,
    diffuse: Rgb,
    specular: Rgb,

    // Phong constants
    k_a: f64,
    k_d: f64,
    k_s: f64,

    // Ray tracing reflection constant, k_r
    reflection: f64,

    // Ray tracing transmission constant, k_t
    transmission: f64,

    // Refraction index of the material
    refraction_index: f64,
}

impl Material {
    pub fn new(
        color: Rgb,
        phong_constants: (f64, f64, f64),
        reflection: f64,
        transmission: f64,
        refraction_index: f64,
    ) -> Material {
        let (k_a, k_d, k_s) = phong_constants;
        Material {
            ambient: &color * AMBIENT_FACTOR,
            diffuse: color,
            specular: SPECULAR_COLOR,
            k_a,
            k_d,
            k_s,
            reflection,
            transmission,
            refraction_index,
        }
    }

    pub fn ambient(&self) -> &Rgb {
        &self.ambient
    }

    pub fn diffuse(&self) -> &Rgb {
        &self.diffuse
    }

    pub fn specular(&self) -> &Rgb {
        &self.specular
    }

    pub fn specular_exponent(&self) -> f64 {
        SHININESS
    }

    pub fn reflection(&self) -> f64 {
        self.reflection
    }

    pub fn transmission(&self) -> f64 {
        self.transmission
    }

    pub fn refraction_index(&self) -> f64 {
        self.refraction_index
    }

    pub fn phong_constants(&self) -> (f64, f64, f64) {
        (self.k_a, self.k_d, self.k_s)
    }
}


// Performs phong shading in a scene
pub fn phong(
    intersect: &Intersect,
    shapes: &Vec<Box<Shape>>,
    lights: &Vec<Light>,
    v: Vector3<f64>,
) -> Rgb {
    let n = intersect.normal;

    let (k_a, k_d, k_s) = intersect.color.phong_constants();

    // Start with the base ambient lighting
    let ambient = intersect.color.ambient() * AMBIENT_FACTOR * k_a;

    lights.iter().fold(ambient, |result, ref light| {
        // Shadow ray
        let s = (light.position - intersect.point).normalize();

        // Reflected vector
        let r = (s - 2.0 * (dot(s, n) / n.magnitude().powi(2)) * n).normalize();

        // Calculate the color including shadow transmission
        let light_color = trace_shadow(intersect.point, intersect.shape, shapes, &light, 1);

        // Calculate diffuse light component
        let diffuse_dot = dot(s, n);
        let diffuse = if diffuse_dot > 0.0 {
            Some((intersect.color.diffuse() * &light_color) * diffuse_dot * k_d)
        } else {
            None
        };

        // Calculate the specular component
        let specular_dot = dot(r, v);
        let specular = if specular_dot > 0.0 {
            Some(
                ((intersect.color.specular() * &light_color)
                    * specular_dot.powf(intersect.color.specular_exponent())) * k_s,
            )
        } else {
            None
        };

        [diffuse, specular]
            .to_vec()
            .into_iter()
            .filter_map(|c| c)
            .fold(result, |result, color| result + color)
    })
}

// Calculates the amount to dim based on transmitted shadows
fn trace_shadow(
    point: Vector3<f64>,
    shape: &Shape,
    shapes: &Vec<Box<Shape>>,
    light: &Light,
    depth: u8,
) -> Rgb {
    let s = (light.position - point).normalize();

    match shape_intersect(&Ray::new(point, s), shapes, Some(shape)) {
        // Nothing blocking, use full value
        None => &light.color * 1.0,

        // If a shape is in the way, check transmission before determining shadow
        Some(blocking) => {
            let k_t = blocking.color.transmission();
            let (_, k_d, _) = blocking.color.phong_constants();

            // Transmission color should only reduce the light color by the
            // diffuse phong constant for the shape.
            let color = Rgb {
                color: blocking
                    .color
                    .diffuse()
                    .color
                    .map(|channel| u8::MAX - ((u8::MAX - channel) as f64 * k_d) as u8),
            };

            if k_t > 0.0 && depth < MAX_SHADOW_DEPTH {
                let entry_v = (light.position - blocking.point).normalize();
                let transmission = transmission_ray(entry_v, &blocking);

                // We were transmitting through the shape, so there should
                // definitely be an exit point
                let exit = blocking.shape.intersect(&transmission).unwrap();

                color * trace_shadow(exit.point, blocking.shape, shapes, light, depth + 1) * k_t
            } else if k_t > 0.0 {
                color * k_t
            } else {
                blocking.color.diffuse() * 0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // Tests multiplying the same color struct
    #[test]
    fn test_rgb_mul_samecolor() {
        let color = Rgb::new([100, 0, 0]);
        assert_eq!(Rgb::new([39, 0, 0]), &color * &color);
        assert_eq!(Rgb::new([15, 0, 0]), &color * &color * &color);
        assert_eq!(Rgb::new([15, 0, 0]), (&color * &color) * &color);
        assert_eq!(Rgb::new([15, 0, 0]), &color * (&color * &color));
        assert_eq!(Rgb::new([5, 0, 0]), (&color * &color) * (&color * &color));
    }

    // Tests multiplying two different colors
    #[test]
    fn test_rgb_mul_differentcolors() {
        let color1 = Rgb::new([100, 0, 0]);
        let color2 = Rgb::new([150, 0, 0]);
        assert_eq!(Rgb::new([58, 0, 0]), &color1 * &color2);
    }

    // Tests multiplying different colors with different channels
    #[test]
    fn test_rgb_mul_differentchannels() {
        let color1 = Rgb::new([100, 0, 0]);
        let color2 = Rgb::new([0, 100, 0]);
        assert_eq!(Rgb::new([0, 0, 0]), &color1 * &color2);

        let color3 = Rgb::new([100, 50, 75]);
        let color4 = Rgb::new([150, 100, 50]);
        assert_eq!(Rgb::new([58, 19, 14]), &color3 * &color4);
    }

    // Tests adding the same color struct
    #[test]
    fn test_rgb_add_samecolor() {
        let color = Rgb::new([100, 0, 0]);
        assert_eq!(Rgb::new([200, 0, 0]), &color + &color);
        assert_eq!(Rgb::new([255, 0, 0]), &color + (&color + &color));
        assert_eq!(Rgb::new([255, 0, 0]), (&color + &color) + &color);
        assert_eq!(Rgb::new([255, 0, 0]), (&color + &color) + (&color + &color));
    }

    // Tests adding two different colors
    #[test]
    fn test_rgb_add_differentcolors() {
        let color1 = Rgb::new([100, 0, 0]);
        let color2 = Rgb::new([150, 0, 0]);
        let color3 = Rgb::new([200, 0, 0]);
        assert_eq!(Rgb::new([250, 0, 0]), &color1 + &color2);
        assert_eq!(Rgb::new([255, 0, 0]), &color1 + &color2 + &color3);
    }

    // Tests adding different colors with different channels
    #[test]
    fn test_rgb_add_differentchannels() {
        let color1 = Rgb::new([100, 0, 0]);
        let color2 = Rgb::new([0, 100, 0]);
        assert_eq!(Rgb::new([100, 100, 0]), &color1 + &color2);

        let color3 = Rgb::new([100, 50, 75]);
        let color4 = Rgb::new([150, 100, 50]);
        assert_eq!(Rgb::new([250, 150, 125]), &color3 + &color4);
    }

    // Tests multiplying a color by a scalar
    #[test]
    fn test_rgb_mul_scalar() {
        let color = Rgb::new([100, 10, 1]);
        assert_eq!(Rgb::new([200, 20, 2]), &color * 2);
        assert_eq!(Rgb::new([255, 30, 3]), &color * 3);
        assert_eq!(Rgb::new([200, 20, 2]), &color * 2.0);
        assert_eq!(Rgb::new([255, 30, 3]), &color * 3.0);
        assert_eq!(Rgb::new([255, 40, 4]), (&color + &color) * 2);
        assert_eq!(Rgb::new([255, 40, 4]), (&color + &color) * 2.0);
    }
}
