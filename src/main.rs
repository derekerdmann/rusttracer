#[macro_use]
extern crate cgmath;
extern crate image;
extern crate piston_window;

mod tracer;
mod sphere;
mod floor;
mod ray;
mod light;

use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use image::ConvertBuffer;
use cgmath::vec3;
use tracer::{Background, Shape};
use sphere::Sphere;
use floor::Floor;
use ray::Ray;
use light::{Light, Material, Rgb};

const IMAGE_PLANE: f64 = 0.5;


fn main() {
    let background = Background {
        color: Rgb::new([0, 175, 215]),
    };

    let sphere1 = Sphere::new(
        vec3(-0.87, -0.5, 2.25),
        0.45,
        Material::new(Rgb::new([179, 179, 179]), (0.15, 0.25, 1.0), 0.75, 0.0, 0.0),
    );

    let sphere2 = Sphere::new(
        vec3(0.0, 0.0, 1.5),
        0.5,
        Material::new(
            Rgb::new([255, 255, 255]),
            (0.075, 0.075, 0.2),
            0.01,
            0.85,
            0.95,
        ),
    );

    let floor = Floor::new(
        vec3(-2.0, -2.0, 0.0),
        vec3(-2.0, 2.0, 0.0),
        vec3(2.0, 2.0, 0.0),
        vec3(2.0, -2.0, 0.0),
        Material::new(Rgb::new([255, 0, 0]), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
        Material::new(Rgb::new([255, 255, 0]), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
    );
    let floor = floor.rotate_x(65.0);
    let floor = floor.translate(vec3(-1.0, -1.25, 2.0));

    let shapes: Vec<&Shape> = vec![&sphere1, &sphere2, &floor];

    let light1 = Light {
        position: vec3(2.0, 3.0, -4.0),
        color: Rgb::new([255, 255, 255]),
    };

    let lights: Vec<&Light> = vec![&light1];

    // Create the raw image buffer
    let mut image = image::RgbImage::from_pixel(640, 640, image::Rgb([255, 0, 0]));

    let height = image.height();

    let dx = 1.0 / image.width() as f64;
    let dy = 1.0 / image.height() as f64;

    // Set up computation channels
    let (compute_tx, compute_rx) = mpsc::channel();

    // Trace through the scene
    for (real_xpixel, real_ypixel, _) in image.enumerate_pixels() {
        // enumerate_pixels_mut() iterates from top to bottom and left to right,
        // rather than bottom to top, left to right. Rather than reworking the
        // ray calculations, just figure out the pixel coordinates we actually
        // want to calculate.
        let xpixel = real_xpixel;
        let ypixel = height - real_ypixel;

        let x = -0.5 + (xpixel as f64) * dx;
        let y = -0.5 + (ypixel as f64) * dy;

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(x, y, IMAGE_PLANE));

        compute_tx.send((real_xpixel, real_ypixel, r)).unwrap();
    }

    loop {
        let result = compute_rx.try_recv();

        match result {
            Ok((xpixel, ypixel, r)) => {
                let pixel = image.get_pixel_mut(xpixel, ypixel);
                *pixel = tracer::illuminate(r, &shapes, &lights, &background, None, 1).color;
            },
            Err(e) => {
                match e {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => panic!("Channel disconnected!"),
                }
            },
        }
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
