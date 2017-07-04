extern crate piston_window;
extern crate image;
extern crate graphics;
extern crate vecmath;

use image::Rgba;
use graphics::math::{ Vec3d, Scalar };

// Objects that can be placed in a scene
struct Background {
    color: Rgba<u8>
}

struct Sphere {
    center: Vec3d,
    r: Scalar,
    color: Rgba<u8>,
}

struct Floor {
    bottomLeft: Vec3d,
    topLeft: Vec3d,
    topRight: Vec3d,
    bottomRight: Vec3d,
    normal: Vec3d,
    f: Scalar,
    color: Rgba<u8>,
}

// Individual ray that is fired through the scene
struct Ray {
    origin: Vec3d,
    direction: Vec3d,
}


// Trait for objects that can be placed in the raytracer scene
trait Traceable {

    // If the Ray intersects the shape, returns the distance from the Ray's
    // origin and the color at that point.
    fn intersect(&self, ray: &Ray) -> Option<(f64, Rgba<u8>)>;
}

// The background object always intersects and returns its static color
impl Traceable for Background {
    fn intersect(&self, _: &Ray) -> Option<(f64, Rgba<u8>)> {
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
    fn intersect(&self, ray: &Ray) -> Option<(f64, Rgba<u8>)> {

        // B=2 * (dx(x_o −x_c)+dy(y_o −y_c)+dz(z_o −z_c)) 
        // which is just the dot product
        // B = 2 * (d . (origin - center))
        let b = 2.0 * vecmath::vec3_dot(ray.direction, vecmath::vec3_sub(ray.origin, self.center));

        // C = (x_o −x_c)^2 +(y_o −y_c)^2 +(z_o −z_c)^2 − r^2
        // which also uses the dot product:
        // tmp = origin - center;
        // C = tmp . tmp - r^2
        let c_sub = vecmath::vec3_sub(ray.origin, self.center);
        let c = vecmath::vec3_dot(c_sub, c_sub) - (self.r * self.r);

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
    fn intersect(&self, ray: &Ray) -> Option<(f64, Rgba<u8>)> {

        let dist = -1.0 * vecmath::vec3_dot(self.normal, ray.origin) + self.f /
                    vecmath::vec3_dot(self.normal, ray.direction);

        if dist > 0.0 {
            let intersect = vecmath::vec3_scale(vecmath::vec3_add(ray.origin, ray.direction), dist);

            // Make sure the value is inside the shape boundaries
            if intersect[0] < self.bottomLeft[0] ||
                intersect[0] > self.bottomRight[0] || 
                intersect[1] < self.bottomLeft[1] || 
                intersect[1] > self.bottomRight[1] {
                None
            } else {
                Some((dist, self.color))
            }

        } else {
            None
        }
    }
}


// Normalizes a vector's magnitude to 1 unit
fn normalize(v: Vec3d) -> Vec3d {
    let s = 1.0f64 / (4.0f64).sqrt();
    vecmath::vec3_scale(v, s)
}


fn main() {

    // Create the raw image buffer
    let mut image = image::RgbaImage::from_pixel(640, 640, Rgba([255, 0, 0, 255]));

    // Set up the window for rendering
    let mut window: piston_window::PistonWindow = 
            piston_window::WindowSettings::new("RustTracer", [640, 640])
            .exit_on_esc(true).build().unwrap();

    // Generate a texture so the image buffer can be rendered to the screen
    let texture = piston_window::Texture::from_image(
            &mut window.factory,
            &image,
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
