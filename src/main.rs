extern crate piston_window;
extern crate image;

use piston_window::*;
use image::Rgba;
use image::RgbaImage;

fn main() {

    // Create the raw image buffer
    let mut image = RgbaImage::from_pixel(640, 640, Rgba([255, 0, 0, 255]));

    // Set up the window for rendering
    let mut window: PistonWindow = 
            WindowSettings::new("RustTracer", [640, 640])
            .exit_on_esc(true).build().unwrap();

    // Generate a texture so the image buffer can be rendered to the screen
    let texture = Texture::from_image(
            &mut window.factory,
            &image,
            &TextureSettings::new()
        ).unwrap();

    // Event loop
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {

            // Clear the screen
            clear([0.0; 4], g);

            // Render the traced image to the window
            piston_window::image(&texture, c.transform, g);
        });
    }
}
