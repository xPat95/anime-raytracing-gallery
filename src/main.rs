mod camera;
mod cube;
mod framebuffer;
mod intersect;
mod material;
mod ray_intersect;
mod render;
mod vec3;

use camera::Camera;
use cube::Cube;
use framebuffer::Framebuffer;
use material::Material;
use raylib::prelude::*;
use vec3::Vec3;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, Color::BLACK);
    let camera = Camera::look_at(Vec3::new(0.0, 1.0, 6.0), Vec3::new(0.0, 0.1, 0.0), 60.0);
    let cubes = build_scene();

    render::render(&mut framebuffer, &camera, &cubes);

    let (mut raylib, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Black Clover Ray Tracing Skull - Cubes")
        .build();

    raylib.set_target_fps(60);

    let image = framebuffer.to_image();
    let texture = raylib
        .load_texture_from_image(&thread, &image)
        .expect("could not create texture from framebuffer");

    while !raylib.window_should_close() {
        let mut drawing = raylib.begin_drawing(&thread);
        drawing.clear_background(Color::BLACK);
        drawing.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}

fn build_scene() -> Vec<Cube> {
    vec![
        Cube::from_center_size(
            Vec3::new(-0.85, -0.35, 0.0),
            1.4,
            Material::new(Color::new(198, 44, 62, 255)),
        ),
        Cube::from_center_size(
            Vec3::new(0.05, -0.20, -0.85),
            1.15,
            Material::new(Color::new(60, 151, 92, 255)),
        ),
        Cube::from_center_size(
            Vec3::new(1.05, -0.45, -0.10),
            0.95,
            Material::new(Color::new(51, 111, 210, 255)),
        ),
        Cube::from_center_size(
            Vec3::new(0.15, 0.90, -1.55),
            0.75,
            Material::new(Color::new(223, 184, 70, 255)),
        ),
    ]
}
