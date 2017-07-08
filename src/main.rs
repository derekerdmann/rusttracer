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

impl Floor {

    fn new(bottomLeft: Vec3d, topLeft: Vec3d, topRight: Vec3d, bottomRight: Vec3d, color: Rgba<u8>) -> Floor {

        let (normal, f) = Floor::calculatenormal(bottomLeft, topLeft, bottomRight);

        Floor {
            bottomLeft, topLeft, topRight, bottomRight,
            normal, f,
            color
        }
    }

    /// Translates the floor by the amount specified in the translation vector
    fn translate(&self, translation: Vec3d) -> Floor {
        Floor::new(
            vecmath::vec3_add(self.bottomLeft, translation),   
            vecmath::vec3_add(self.topLeft, translation),
            vecmath::vec3_add(self.topRight, translation),
            vecmath::vec3_add(self.bottomRight, translation),
            self.color
        )
    }

    /// Rotates the floor around the X axis by the provided rotation in degrees
    fn rotate_x(&self, rotation: Scalar) -> Floor {

        /// Rotates a single vector
        fn rotate_x(v: Vec3d) -> Vec3d {
            let theta = std::f64::consts::PI / 180.0;
        
            [
                v[0],
                v[1] * theta.cos() + v[2] * -theta.sin(),
                v[1] * theta.sin() + v[2] * theta.cos()
            ]
        }
        
        Floor::new(
            rotate_x(self.bottomLeft),
            rotate_x(self.topLeft),
            rotate_x(self.topRight),
            rotate_x(self.bottomRight),
            self.color
        )
    }

    // Given 3 corners, calculates the normal and constant F
    fn calculatenormal(c1: Vec3d, c2: Vec3d, c3: Vec3d) -> (Vec3d, Scalar) {

        let a = vecmath::vec3_sub(c1, c2);
        let b = vecmath::vec3_sub(c1, c3);

        let normal: Vec3d = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0]
        ];

        (vecmath::vec3_normalized(normal), -vecmath::vec3_dot(normal, c1))
    }
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
    fn intersect(&self, ray: &Ray) -> Option<(Scalar, Rgba<u8>)>;
}

// The background object always intersects and returns its static color
impl Traceable for Background {
    fn intersect(&self, _: &Ray) -> Option<(Scalar, Rgba<u8>)> {
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
    fn intersect(&self, ray: &Ray) -> Option<(Scalar, Rgba<u8>)> {

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
    fn intersect(&self, ray: &Ray) -> Option<(Scalar, Rgba<u8>)> {

        let dist = -vecmath::vec3_dot(self.normal, ray.origin) + self.f /
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


fn main() {

    let background = Background { color: Rgba([0, 175, 215, 255]) };

    let sphere1 = Sphere {
        center: [-0.75, -0.5, 2.25],
        r: 0.45,
        color: Rgba([255, 255, 0, 255])
        };

    let sphere2 = Sphere {
        center: [0.0, 0.0, 1.5],
        r: 0.5,
        color: Rgba([0, 225, 0, 255])
        };

    let floor = Floor::new(
        [ -2.0, -2.0, 0.0],
        [ -2.0, 2.0, 0.0],
        [ 2.0, 2.0, 0.0 ],
        [ 2.0, -2.0, 0.0 ],
        Rgba([255, 0, 0, 255])
    );
    let floor = floor.rotate_x( 75.0 );
    let floor = floor.translate( [-1.0, -1.25, 2.0] );

    let shapes: Vec<&Traceable> = vec![&sphere1, &sphere2, &floor, &background];

    // Create the raw image buffer
    let mut image = image::RgbaImage::from_pixel(640, 640, Rgba([255, 0, 0, 255]));

    // Trace through the scene
    //TODO build tracer

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
