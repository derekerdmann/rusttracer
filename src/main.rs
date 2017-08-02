extern crate piston_window;
extern crate image;
extern crate graphics;
extern crate cgmath;

use image::{ Rgb, ConvertBuffer };
use cgmath::{ Vector3, InnerSpace, dot, vec3 };

const IMAGE_PLANE: f64 = 0.5;

// Objects that can be placed in a scene
struct Background {
    color: Rgb<u8>
}

struct Sphere {
    center: Vector3<f64>,
    r: f64,
    color: Rgb<u8>,
}

struct Floor {
    bottom_left: Vector3<f64>,
    top_left: Vector3<f64>,
    top_right: Vector3<f64>,
    bottom_right: Vector3<f64>,
    normal: Vector3<f64>,
    f: f64,
    color: Rgb<u8>,
}

impl Floor {

    fn new(bottom_left: Vector3<f64>, top_left: Vector3<f64>, top_right: Vector3<f64>, bottom_right: Vector3<f64>, color: Rgb<u8>) -> Floor {

        // Given 3 of the corners, calculate the normal and F
        let a = bottom_left - top_left;
        let b = bottom_left - bottom_right;

        let normal = Vector3 {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x
        }.normalize();

        Floor {
            bottom_left, top_left, top_right, bottom_right,
            normal,
            f: -dot(normal, bottom_left),
            color
        }
    }

    /// Translates the floor by the amount specified in the translation vector
    fn translate(&self, translation: Vector3<f64>) -> Floor {
        Floor::new(
            self.bottom_left + translation,
            self.top_left + translation,
            self.top_right + translation,
            self.bottom_right + translation,
            self.color
        )
    }

    /// Rotates the floor around the X axis by the provided rotation in degrees
    fn rotate_x(&self, rotation: f64) -> Floor {

        /// Rotates a single vector
        fn rotate_x(v: Vector3<f64>, rotation: f64) -> Vector3<f64> {

            let theta = rotation * (std::f64::consts::PI / 180.0);
        
            Vector3 {
                x: v.x,
                y: v.y * theta.cos() + v.z * -theta.sin(),
                z: v.y * theta.sin() + v.z * theta.cos()
            }
        }
        
        Floor::new(
            rotate_x(self.bottom_left, rotation),
            rotate_x(self.top_left, rotation),
            rotate_x(self.top_right, rotation),
            rotate_x(self.bottom_right, rotation),
            self.color
        )
    }
}

// Individual ray that is fired through the scene
struct Ray {
    origin: Vector3<f64>,
    direction: Vector3<f64>,
}

impl Ray {
    fn new(origin: Vector3<f64>, direction: Vector3<f64>) -> Ray {
        Ray { origin: origin, direction: direction.normalize() }
    }
}


// Trait for objects that can be placed in the raytracer scene
trait Traceable {

    // If the Ray intersects the shape, returns the distance from the Ray's
    // origin and the color at that point.
    fn intersect(&self, ray: &Ray) -> Option<(f64, Rgb<u8>)>;
}

// The background object always intersects and returns its static color
impl Traceable for Background {
    fn intersect(&self, _: &Ray) -> Option<(f64, Rgb<u8>)> {
        Some((std::f64::INFINITY, self.color))
    }
}


impl Traceable for Sphere {

    /// Sphere intersection formula comes from CG II slides
    /// (2-2b-rt-basics-4.pdf). \omega is the distance from the origin of the ray
    /// to the intersect point.
    ///
    /// \omega = (-B \pm \sqrt{B^2 - 4 * C}) / 2
    ///
    fn intersect(&self, ray: &Ray) -> Option<(f64, Rgb<u8>)> {

        // B=2 * (dx(x_o −x_c)+dy(y_o −y_c)+dz(z_o −z_c)) 
        // which is just the dot product
        // B = 2 * (d . (origin - center))
        let b = 2.0 * dot(ray.direction, ray.origin - self.center);

        // C = (x_o −x_c)^2 +(y_o −y_c)^2 +(z_o −z_c)^2 − r^2
        // which also uses the dot product:
        // tmp = origin - center;
        // C = tmp . tmp - r^2
        let c_sub = ray.origin - self.center;
        let c = dot(c_sub, c_sub) - (self.r * self.r);

        // Partial quadratic solution
        let partial = b * b - 4.0 * c;

        if partial < 0.0 {
            None

        } else {
            let d1 = ( -b + partial ) / 2.0;
            let d2 = ( -b - partial ) / 2.0;

            // There are two solutions, so return the smallest positive result.
            // The larger value would be the far side of the sphere.
            let d = if d1 < 0.0 {
                d2
            } else if d2 < 0.0 {
                d1
            } else {
                f64::min(d1,d2)
            };

            Some((d, self.color))
        }
    }
}


impl Traceable for Floor {

    /// Plane intersection formula comes from CG II slides
    /// (2-2b-rt-basics-4.pdf).
    /// \omega = -(P_n . P_o + F) / (P+n . D)
    fn intersect(&self, ray: &Ray) -> Option<(f64, Rgb<u8>)> {

        let dist = -(dot(self.normal, ray.origin) + self.f) /
                    dot(self.normal, ray.direction);

        if dist > 0.0 {

            let intersect = ray.origin + (ray.direction * dist);
            // Make sure the value is inside the shape boundaries
            if intersect[0] >= self.bottom_left[0] && intersect[0] <= self.bottom_right[0] &&
                intersect[1] >= self.bottom_left[1] && intersect[1] <= self.top_left[1] {

                Some((dist, self.color))
            } else {
                None
            }

        } else {
            None
        }
    }
}


fn main() {

    let background = Background { color: Rgb([0, 175, 215]) };

    let sphere1 = Sphere {
        center: vec3(-0.75, -0.5, 2.25),
        r: 0.45,
        color: Rgb([255, 255, 0])
        };

    let sphere2 = Sphere {
        center: vec3(0.0, 0.0, 1.5),
        r: 0.5,
        color: Rgb([0, 225, 0])
        };

    let floor = Floor::new(
        vec3(-2.0, -2.0, 0.0),
        vec3(-2.0, 2.0, 0.0),
        vec3(2.0, 2.0, 0.0),
        vec3(2.0, -2.0, 0.0),
        Rgb([255, 0, 0])
    );
    let floor = floor.rotate_x(75.0);
    let floor = floor.translate(vec3(-1.0, -1.25, 2.0));

    let shapes: Vec<&Traceable> = vec![&sphere1, &sphere2, &floor];

    // Create the raw image buffer
    let mut image = image::RgbImage::from_pixel(640, 640, Rgb([255, 0, 0]));

    let height = image.height(); // TODO properly translate, instead of hack

    let dx = 1.0 / image.width() as f64;
    let dy = 1.0 / image.height() as f64;

    // Trace through the scene
    for (xpixel, ypixel, pixel) in image.enumerate_pixels_mut() {

        let ypixel = height - ypixel; //TODO properly translate

        let x = -0.5 + (xpixel as f64) * dx;
        let y = -0.5 + (ypixel as f64) * dy;

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(x, y, IMAGE_PLANE));

        // Calculate the color for the pixel
        let bg = background.intersect(&r).expect("Background must always intersect!");
        let (_, color) = shapes.iter().fold(bg, { |(best_dist, best_color), &shape|
            match shape.intersect(&r) {
                Some((dist, color)) if dist < best_dist => (dist, color),
                _ => (best_dist, best_color)
            }
        });

        // Update the pixel color
        *pixel = color;
    }

    // Set up the window for rendering
    let mut window: piston_window::PistonWindow = 
            piston_window::WindowSettings::new("RustTracer", [640, 640])
            .exit_on_esc(true).build().unwrap();

    // Generate a texture so the image buffer can be rendered to the screen
    let texture = piston_window::Texture::from_image(
            &mut window.factory,
            &image.convert(),
            &piston_window::TextureSettings::new()
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
