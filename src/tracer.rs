extern crate std;

use cgmath::{dot, InnerSpace, Vector3};
use ray::Ray;
use light::{phong, Light, Material, Rgb};
use std::any::Any;

const MAX_DEPTH: u8 = 5;
const ETA_AIR: f64 = 1.0;

// Represents the intersection of a Ray with an object
pub struct Intersect<'a> {
    // Distance from the origin where the intersect occurs
    pub distance: f64,

    // Point in space where the intersect occurs
    pub point: Vector3<f64>,

    // Normal vector from the surface of the shape at this intersect
    pub normal: Vector3<f64>,

    // Material of the object where the intersect occurs
    pub color: &'a Material,

    // Shape that the ray intersects
    pub shape: &'a Shape,
}

// Trait for objects that can be placed in the raytracer scene
pub trait Shape {
    // If the Ray intersects the shape, returns the distance from the Ray's
    // origin and the color at that point.
    fn intersect(&self, ray: &Ray) -> Option<Intersect>;

    // Used to downcast and check equality
    fn eq(&self, other: &Shape) -> bool;
    fn as_any(&self) -> &Any;
}

impl<'a, 'b> PartialEq<Shape + 'b> for Shape + 'a {
    fn eq(&self, other: &(Shape + 'b)) -> bool {
        Shape::eq(self, other)
    }
}


// Objects that can be placed in a scene
pub struct Background {
    pub color: Rgb,
}

// Of all shapes that intersect with this ray, select the closest one that's in
// front of the starting point.
pub fn shape_intersect<'a>(
    r: &Ray,
    shapes: &Vec<&'a Shape>,
    exclude: Option<&Shape>,
) -> Option<Intersect<'a>> {
    shapes
        .iter()
        .filter(|&shape| exclude.map_or(true, |e| &e != shape))
        .filter_map(|&shape| shape.intersect(&r))
        .filter(|intersect| intersect.distance >= 0.0)
        .min_by(|first, second| {
            first.distance.partial_cmp(&second.distance).unwrap()
        })
}

// The main tracer function. Fires the ray into the scene, calculating the
// objects it intersects and the final output color
pub fn illuminate(
    r: Ray,
    shapes: &Vec<&Shape>,
    lights: &Vec<&Light>,
    background: &Background,
    last_shape: Option<&Shape>,
    depth: u8,
) -> Rgb {
    match shape_intersect(&r, shapes, last_shape) {
        Some(intersect) => {
            let k_r = intersect.color.reflection();
            let k_t = intersect.color.transmission();

            let local = phong(
                &intersect,
                shapes,
                lights,
                (r.direction() - r.origin).normalize(),
            );

            let reflection = if depth < MAX_DEPTH && k_r > 0.0 {
                Some(reflect(&intersect, depth, shapes, lights, background) * k_r)
            } else {
                None
            };

            let transmission = if depth < MAX_DEPTH && k_t > 0.0 {
                Some(transmit(r.direction(), &intersect, depth, shapes, lights, background) * k_t)
            } else {
                None
            };

            [reflection, transmission]
                .to_vec()
                .into_iter()
                .filter_map(|c| c)
                .fold(local, |result, color| result + color)
        }
        None => background.color.clone(),
    }
}

fn reflect(
    intersect: &Intersect,
    depth: u8,
    shapes: &Vec<&Shape>,
    lights: &Vec<&Light>,
    background: &Background,
) -> Rgb {
    let i = intersect.point;
    let n = intersect.normal;
    let r = i - 2.0 * (n * dot(i, n));

    let ray = Ray::new(intersect.point, r);

    illuminate(
        ray,
        shapes,
        lights,
        background,
        Some(intersect.shape),
        depth + 1,
    )
}

fn transmit(
    d: Vector3<f64>,
    intersect: &Intersect,
    depth: u8,
    shapes: &Vec<&Shape>,
    lights: &Vec<&Light>,
    background: &Background,
) -> Rgb {
    let in_shape = dot(-d, intersect.normal) < 0.0;

    let (n, n_it) = if in_shape {
        (
            -intersect.normal,
            intersect.color.refraction_index() / ETA_AIR,
        )
    } else {
        (
            intersect.normal,
            ETA_AIR / intersect.color.refraction_index(),
        )
    };

    // Negative discriminant indicates total internal reflection
    let discriminant = 1.0 + (n_it * n_it * (dot(-d, n) * dot(-d, n) - 1.0));

    let t = if discriminant < 0.0 {
        d - 2.0 * (n * dot(d, n))
    } else {
        (d * n_it) + (n * (n_it * dot(-d, n) - discriminant.sqrt()))
    };

    let ray = Ray::new(intersect.point, t);

    illuminate(
        ray,
        shapes,
        lights,
        background,
        Some(intersect.shape),
        depth + 1,
    )
}



#[cfg(test)]
mod tests {

    use cgmath::vec3;
    use ray::Ray;
    use tracer::Shape;
    use floor::Floor;
    use light::{Material, Rgb};
    use super::shape_intersect;

    // Tests that the closest shape is selected
    #[test]
    fn intersect_ordering() {
        let color1 = Rgb::new([255, 0, 0]);
        let color2 = Rgb::new([0, 255, 0]);

        let f1 = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Material::new(color1.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
            Material::new(color1.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
        );

        let f2 = Floor::new(
            vec3(-1.0, -1.0, 2.0),
            vec3(-1.0, 1.0, 2.0),
            vec3(1.0, -1.0, 2.0),
            vec3(1.0, 1.0, 2.0),
            Material::new(color2.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
            Material::new(color2.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
        );

        let shapes: Vec<&Shape> = vec![&f1, &f2];

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));

        let intersect =
            shape_intersect(&r, &shapes, None).expect("Both of these objects should intersect");

        assert_ulps_eq!(1.0, intersect.distance);
    }

    // Tests that a shape is excluded if specified
    #[test]
    fn intersect_exclude() {
        let color1 = Rgb::new([255, 0, 0]);
        let color2 = Rgb::new([0, 255, 0]);

        let f1 = Floor::new(
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            Material::new(color1.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
            Material::new(color1.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
        );

        let f2 = Floor::new(
            vec3(-1.0, -1.0, 2.0),
            vec3(-1.0, 1.0, 2.0),
            vec3(1.0, -1.0, 2.0),
            vec3(1.0, 1.0, 2.0),
            Material::new(color2.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
            Material::new(color2.clone(), (1.0, 1.0, 1.0), 0.0, 0.0, 0.0),
        );

        let shapes: Vec<&Shape> = vec![&f1, &f2];

        let r = Ray::new(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0));

        // Exclude a shape that already isn't closest
        let intersect = shape_intersect(&r, &shapes, Some(&f2)).expect("f1 should intersect");
        assert_ulps_eq!(1.0, intersect.distance);

        // Exclude the closest shape
        let intersect = shape_intersect(&r, &shapes, Some(&f1)).expect("f2 should intersect");
        assert_ulps_eq!(2.0, intersect.distance);

        // Exclude the only shape
        let shapes: Vec<&Shape> = vec![&f1];
        let intersect = shape_intersect(&r, &shapes, Some(&f1));
        assert!(intersect.is_none());
    }

}
