extern crate piston_window;
extern crate image;
extern crate graphics;
extern crate vecmath;

use image::{ Rgb, ConvertBuffer };
use graphics::math::{ Vec3d, Scalar };
use vecmath::{ vec3_add, vec3_dot, vec3_scale, vec3_sub, vec3_normalized };

const IMAGE_PLANE: Scalar = 0.5;

// Objects that can be placed in a scene
struct Background {
    color: Rgb<u8>
}

struct Sphere {
    center: Vec3d,
    r: Scalar,
    color: Rgb<u8>,
}

struct Floor {
    bottom_left: Vec3d,
    top_left: Vec3d,
    top_right: Vec3d,
    bottom_right: Vec3d,
    normal: Vec3d,
    f: Scalar,
    color: Rgb<u8>,
}

impl Floor {

    fn new(bottom_left: Vec3d, top_left: Vec3d, top_right: Vec3d, bottom_right: Vec3d, color: Rgb<u8>) -> Floor {

        // Given 3 of the corners, calculate the normal and F
        let a = vec3_sub(bottom_left, top_left);
        let b = vec3_sub(bottom_left, bottom_right);

        let normal: Vec3d = vec3_normalized([
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0]
        ]);

        Floor {
            bottom_left, top_left, top_right, bottom_right,
            normal,
            f: -vec3_dot(normal, bottom_left),
            color
        }
    }

    /// Translates the floor by the amount specified in the translation vector
    fn translate(&self, translation: Vec3d) -> Floor {
        Floor::new(
            vec3_add(self.bottom_left, translation),   
            vec3_add(self.top_left, translation),
            vec3_add(self.top_right, translation),
            vec3_add(self.bottom_right, translation),
            self.color
        )
    }

    /// Rotates the floor around the X axis by the provided rotation in degrees
    fn rotate_x(&self, rotation: Scalar) -> Floor {

        /// Rotates a single vector
        fn rotate_x(v: Vec3d, rotation: Scalar) -> Vec3d {

            let theta = rotation * (std::f64::consts::PI / 180.0);
        
            [
                v[0],
                v[1] * theta.cos() + v[2] * -theta.sin(),
                v[1] * theta.sin() + v[2] * theta.cos()
            ]
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
    origin: Vec3d,
    direction: Vec3d,
}

impl Ray {
    fn new(origin: Vec3d, direction: Vec3d) -> Ray {
        Ray { origin: origin, direction: vec3_normalized(direction) }
    }
}


// Trait for objects that can be placed in the raytracer scene
trait Traceable {

    // If the Ray intersects the shape, returns the distance from the Ray's
    // origin and the color at that point.
    fn intersect(&self, ray: &Ray) -> Option<(Scalar, Rgb<u8>)>;
}

// The background object always intersects and returns its static color
impl Traceable for Background {
    fn intersect(&self, _: &Ray) -> Option<(Scalar, Rgb<u8>)> {
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
    fn intersect(&self, ray: &Ray) -> Option<(Scalar, Rgb<u8>)> {

        // B=2 * (dx(x_o −x_c)+dy(y_o −y_c)+dz(z_o −z_c)) 
        // which is just the dot product
        // B = 2 * (d . (origin - center))
        let b = 2.0 * vec3_dot(ray.direction, vec3_sub(ray.origin, self.center));

        // C = (x_o −x_c)^2 +(y_o −y_c)^2 +(z_o −z_c)^2 − r^2
        // which also uses the dot product:
        // tmp = origin - center;
        // C = tmp . tmp - r^2
        let c_sub = vec3_sub(ray.origin, self.center);
        let c = vec3_dot(c_sub, c_sub) - (self.r * self.r);

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
                Scalar::min(d1,d2)
            };

            Some((d, self.color))
        }
    }
}


impl Traceable for Floor {

    /// Plane intersection formula comes from CG II slides
    /// (2-2b-rt-basics-4.pdf).
    /// \omega = -(P_n . P_o + F) / (P+n . D)
    fn intersect(&self, ray: &Ray) -> Option<(Scalar, Rgb<u8>)> {

        let dist = -(vec3_dot(self.normal, ray.origin) + self.f) /
                    vec3_dot(self.normal, ray.direction);

        if dist > 0.0 {

            let intersect = vec3_add(ray.origin, vec3_scale(ray.direction, dist));

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
        center: [-0.75, -0.5, 2.25],
        r: 0.45,
        color: Rgb([255, 255, 0])
        };

    let sphere2 = Sphere {
        center: [0.0, 0.0, 1.5],
        r: 0.5,
        color: Rgb([0, 225, 0])
        };

    let floor = Floor::new(
        [-2.0, -2.0, 0.0],
        [-2.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
        [2.0, -2.0, 0.0],
        Rgb([255, 0, 0])
    );
    let floor = floor.rotate_x( 75.0 );
    let floor = floor.translate( [-1.0, -1.25, 2.0] );

    let shapes: Vec<&Traceable> = vec![&sphere1, &sphere2, &floor];

    // Create the raw image buffer
    let mut image = image::RgbImage::from_pixel(640, 640, Rgb([255, 0, 0]));

    let height = image.height(); // TODO properly translate, instead of hack

    let dx = 1.0 / image.width() as Scalar;
    let dy = 1.0 / image.height() as Scalar;

    // Trace through the scene
    for (xpixel, ypixel, pixel) in image.enumerate_pixels_mut() {

        let ypixel = height - ypixel; //TODO properly translate

        if (xpixel == 0 && ypixel == 0) || (xpixel == 100 && ypixel == 100){
            *pixel = Rgb([0, 0, 0]);
            continue;
        }

        let x = -0.5 + (xpixel as Scalar) * dx;
        let y = -0.5 + (ypixel as Scalar) * dy;

        let r = Ray::new([0.0, 0.0, 0.0], [x, y, IMAGE_PLANE]);

        // Calculate the color for the pixel
        let bg = background.intersect(&r).expect("Background must always intersect!");
        let (_, color) = shapes.iter().fold(bg, { |(best_dist, best_color), &shape|
            match shape.intersect(&r) {
                Some((dist, color)) => if dist < best_dist {
                    (dist, color)
                } else {
                    (best_dist, best_color)
                },
                None => (best_dist, best_color)
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
