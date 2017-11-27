extern crate piston_window;
extern crate image;

#[allow(unused_imports)]
// this macro_use is only used in test modules
#[macro_use]
extern crate cgmath;

mod tracer;
mod sphere;
mod floor;
mod ray;
mod material;

use image::{Rgb, ConvertBuffer};
use cgmath::vec3;
use tracer::{Shape, Background};
use sphere::Sphere;
use floor::Floor;
use ray::Ray;
use material::{SolidColorMaterial};
use std::rc::Rc;

const IMAGE_PLANE: f64 = 0.5;


fn main() {

    let background = Background { color: Rgb([0, 175, 215]) };

    let sphere1 = Sphere {
        center: vec3(-0.75, -0.5, 2.25),
        r: 0.45,
        material: Box::new(SolidColorMaterial::new(Rgb([255, 255, 0]))),
    };

    let sphere2 = Sphere {
        center: vec3(0.0, 0.0, 1.5),
        r: 0.5,
        material: Box::new(SolidColorMaterial::new(Rgb([0, 225, 0]))),
    };

    let floor = Floor::new(
        vec3(-2.0, -2.0, 0.0),
        vec3(-2.0, 2.0, 0.0),
        vec3(2.0, 2.0, 0.0),
        vec3(2.0, -2.0, 0.0),
        Rc::new(SolidColorMaterial::new(Rgb([255, 0, 0]))),
    );
    let floor = floor.rotate_x(75.0);
    let floor = floor.translate(vec3(-1.0, -1.25, 2.0));

    let shapes: Vec<&Shape> = vec![&sphere1, &sphere2, &floor];

    // Create the raw image buffer
    let mut image = image::RgbImage::from_pixel(640, 640, Rgb([255, 0, 0]));

    let height = image.height();

    let dx = 1.0 / image.width() as f64;
    let dy = 1.0 / image.height() as f64;

    // Trace through the scene
    for (xpixel, ypixel, pixel) in image.enumerate_pixels_mut() {

        // enumerate_pixels_mut() iterates from top to bottom and left to right,
        // rather than bottom to top, left to right. Rather than reworking the
        // ray calculations, just figure out the pixel coordinates we actually
        // want to calculate.
        let ypixel = height - ypixel;

        let x = -0.5 + (xpixel as f64) * dx;
        let y = -0.5 + (ypixel as f64) * dy;

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(x, y, IMAGE_PLANE));

        *pixel = tracer::trace(r, &shapes, &background);
    }

    // Set up the window for rendering
    let mut window: piston_window::PistonWindow =
        piston_window::WindowSettings::new("RustTracer", [640, 640])
            .exit_on_esc(true)
            .build()
            .unwrap();

    // Generate a texture so the image buffer can be rendered to the screen
    let texture = piston_window::Texture::from_image(
        &mut window.factory,
        &image.convert(),
        &piston_window::TextureSettings::new(),
    ).unwrap();

    // Event loop
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {

            // Clear the screen
            piston_window::clear([0.0; 4], g);

            // Render the traced image to the window
            piston_window::image(&texture, c.transform, g);
        });
    }
}
