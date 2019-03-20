mod camera;
mod geom;
mod material;
mod ray;

use rand::prelude::*;
use ray::Ray;
use rayon::prelude::*;

pub use nalgebra_glm as glm;
pub type Vec3 = glm::TVec3<f32>;

use geom::*;
use material::*;

fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

fn trace(r: &Ray, scene: &Scene, depth: usize) -> Vec3 {
    if depth == 0 {
        return glm::zero();
    }
    if let Some(result) = scene.trace(r, 0.001, std::f32::MAX) {
        let material = result.material;
        let reflectance = {
            let n = result.hit.normal;
            let (bounce, pdf) = material.bounce(&r, &result.hit);
            let lambert = material.color / std::f32::consts::PI;
            let costheta = f32::max(glm::dot(&n, &bounce.direction), 0.0);
            let incident = trace(&bounce, scene, depth - 1);
            let (brdf, ks) = material.brdf(&(-r.direction), &bounce.direction, &n);
            let kd = glm::vec3(1.0, 1.0, 1.0) - ks * (1.0 - material.metalness);
            (kd.component_mul(&lambert) + brdf).component_mul(&incident) * costheta / pdf
        };
        reflectance + material.emission
    } else {
        let dir = glm::normalize(&r.direction);
        let t = 0.5 * (dir.y + 1.0);
        let white = glm::vec3(1.0, 1.0, 1.0);
        let azure = glm::vec3(0.5, 0.7, 1.0);
        (1.0 - t) * white + t * azure
    }
}

fn setup_scene<'a>() -> Scene<'a> {
    let mut scene = Scene::new();

    let white = glm::vec3(1.0, 1.0, 1.0);
    let red = glm::vec3(1.0, 0.0, 0.0);
    let green = glm::vec3(0.0, 0.1, 0.0);
    let blue = glm::vec3(0.0, 0.0, 1.0);
    let pink = glm::vec3(0.8, 0.2, 0.2);

    scene.add(Object::new(
        Sphere::new(glm::vec3(1005.0, 2.0, 0.0), 1000.0),
        Material {
            color: red,
            metalness: 0.0,
            roughness: 1.0,
            emission: glm::zero(),
        },
    ));
    scene.add(Object::new(
        Sphere::new(glm::vec3(-1005.0, 2.0, 0.0), 1000.0),
        Material {
            color: blue,
            metalness: 0.0,
            roughness: 1.0,
            emission: glm::zero(),
        },
    ));
    scene.add(Object::new(
        Sphere::new(glm::vec3(0.0, 4.0, 0.0), 1.5),
        Material {
            color: white,
            metalness: 0.0,
            roughness: 1.0,
            emission: white * 10.0,
        },
    ));
    scene.add(Object::new(
        Sphere::new(glm::vec3(0.0, 1005.0, 0.0), 1000.0),
        Material {
            color: white,
            metalness: 0.0,
            roughness: 1.0,
            emission: glm::zero(),
        },
    ));
    scene.add(Object::new(
        Sphere::new(glm::vec3(0.0, -1003.0, 0.0), 1000.0),
        Material {
            color: white,
            metalness: 0.0,
            roughness: 1.0,
            emission: glm::zero(),
        },
    ));
    scene.add(Object::new(
        Sphere::new(glm::vec3(0.0, 0.0, 1005.0), 1000.0),
        Material {
            color: white,
            metalness: 0.0,
            roughness: 1.0,
            emission: glm::zero(),
        },
    ));
    scene.add(Object::new(
        Sphere::new(glm::vec3(0.0, 0.0, -1006.0), 1000.0),
        Material {
            color: green,
            metalness: 0.0,
            roughness: 1.0,
            emission: glm::zero(),
        },
    ));
    scene.add(Object::new(
        Sphere::new(glm::vec3(-2.0, -2.0, 0.0), 1.0),
        Material {
            color: pink,
            metalness: 1.0,
            roughness: 0.2,
            emission: glm::zero(),
        },
    ));
    scene.add(Object::new(
        Sphere::new(glm::vec3(2.0, -2.0, 0.0), 1.0),
        Material {
            color: pink,
            metalness: 0.0,
            roughness: 1.0,
            emission: glm::zero(),
        },
    ));
    scene
}

fn main() {
    let w = 800;
    let h = 800;
    let ss = 200;
    let gamma = 2.2;

    let camera = camera::Camera::looking_at(
        glm::vec3(0.0, 0.0, 5.0),
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        80.0,
        w as f32 / h as f32,
    );
    let scene = setup_scene();

    let buffer = (0..w * h)
        .into_par_iter()
        .flat_map(|i| {
            let x = i % w;
            let y = i / w;
            let color = (0..ss)
                .into_par_iter()
                .map(|_| {
                    let mut rng = rand::thread_rng();
                    let rand: f32 = rng.gen();
                    let u = (x as f32 + rand) / w as f32;
                    let rand: f32 = rng.gen();
                    let v = (y as f32 + rand) / h as f32;
                    let ray = camera.ray_at(u, v);
                    trace(&ray, &scene, 3)
                })
                .sum::<Vec3>()
                / ss as f32;
            vec![
                (clamp(color.x.powf(1.0 / gamma), 0.0, 1.0) * 255.99) as u8,
                (clamp(color.y.powf(1.0 / gamma), 0.0, 1.0) * 255.99) as u8,
                (clamp(color.z.powf(1.0 / gamma), 0.0, 1.0) * 255.99) as u8,
            ]
        })
        .collect::<Vec<_>>();
    image::save_buffer("image.png", &buffer, w, h, image::RGB(8)).unwrap()
}