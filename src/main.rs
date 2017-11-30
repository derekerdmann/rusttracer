#[macro_use]
extern crate cgmath;
extern crate chan;
extern crate image;
extern crate piston_window;
extern crate time;

mod tracer;
mod sphere;
mod floor;
mod ray;
mod light;

use std::sync::Arc;
use std::thread;
use std::sync::mpsc;
use image::ConvertBuffer;
use cgmath::vec3;
use tracer::{Background, Shape};
use sphere::Sphere;
use floor::Floor;
use ray::Ray;
use light::{Light, Material, Rgb};

const IMAGE_PLANE: f64 = 0.5;


fn main() {
    let background = Arc::new(Background {
        color: Rgb::new([0, 175, 215]),
    });

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

    let shapes: Arc<Vec<Box<Shape>>> =
        Arc::new(vec![Box::new(sphere1), Box::new(sphere2), Box::new(floor)]);

    let light1 = Light {
        position: vec3(2.0, 3.0, -4.0),
        color: Rgb::new([255, 255, 255]),
    };

    let lights: Arc<Vec<Light>> = Arc::new(vec![light1]);

    // Create the raw image buffer
    let mut image = image::RgbImage::from_pixel(640, 640, image::Rgb([255, 0, 0]));

    let height = image.height();

    let dx = 1.0 / image.width() as f64;
    let dy = 1.0 / image.height() as f64;

    // Set up computation channel
    let (compute_tx, compute_rx) = chan::async();

    // Set up color result channel
    let (color_tx, color_rx) = mpsc::channel();

    // Measure render speed
    let start = time::precise_time_ns();

    // Queue up all the pixels whose color needs to be calculated
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

        compute_tx.send((real_xpixel, real_ypixel, r));
    }
    drop(compute_tx);

    // Calculate colors
    let mut workers = vec![];
    for _ in 0..4 {
        let rx = compute_rx.clone();
        let tx = color_tx.clone();

        let s = Arc::clone(&shapes);
        let l = Arc::clone(&lights);
        let bg = Arc::clone(&background);
        workers.push(thread::spawn(move || loop {
            match rx.recv() {
                Some((xpixel, ypixel, r)) => {
                    let color = tracer::illuminate(r, &s, &l, &bg, None, 1).color;
                    tx.send((xpixel, ypixel, color)).unwrap();
                }
                None => break,
            }
        }));
    }

    let num_workers = workers.len();

    // Wait for worker threads to finish
    for worker in workers {
        worker.join().unwrap();
    }

    let compute = time::precise_time_ns();

    // Render pixels
    loop {
        match color_rx.try_recv() {
            Ok((xpixel, ypixel, color)) => {
                *image.get_pixel_mut(xpixel, ypixel) = color;
            }
            Err(_) => break,
        }
    }

    let render = time::precise_time_ns();

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

    let display = time::precise_time_ns();

    println!("Number of threads: {}", num_workers);
    println!("Time to compute pixels: {} ms", (compute - start) / 1000000);
    println!("Time to render pixels: {} ms", (render - compute) / 1000000);
    println!("Time to display result: {} ms", (display - render) / 1000000);

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
