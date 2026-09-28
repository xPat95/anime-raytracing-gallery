mod camera;
mod cube;
mod framebuffer;
mod intersect;
mod material;
mod ray_intersect;
mod render;
mod texture;
mod vec3;

use camera::Camera;
use cube::Cube;
use framebuffer::Framebuffer;
use material::Material;
use raylib::prelude::*;
use texture::Texture;
use vec3::Vec3;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

const STONE: usize = 0;
const MOSS_BLOCK: usize = 1;
const OBSIDIAN: usize = 2;
const BLACKSTONE: usize = 3;
const SMOOTH_QUARTZ: usize = 4;

fn main() -> Result<(), String> {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, Color::BLACK);
    let camera = Camera::look_at(Vec3::new(0.0, 2.1, 7.0), Vec3::new(0.0, -0.15, -0.25), 55.0);
    let textures = load_textures()?;
    let cubes = build_scene();

    render::render(&mut framebuffer, &camera, &cubes, &textures);

    let (mut raylib, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Black Clover Ray Tracing Skull - Textured Cubes")
        .build();

    raylib.set_target_fps(60);

    let image = framebuffer.to_image();
    let texture = raylib
        .load_texture_from_image(&thread, &image)
        .map_err(|error| format!("could not create texture from framebuffer: {error}"))?;

    while !raylib.window_should_close() {
        let mut drawing = raylib.begin_drawing(&thread);
        drawing.clear_background(Color::BLACK);
        drawing.draw_texture(&texture, 0, 0, Color::WHITE);
    }

    Ok(())
}

fn load_textures() -> Result<Vec<Texture>, String> {
    [
        "assets/textures/stone.png",
        "assets/textures/moss_block.png",
        "assets/textures/obsidian.png",
        "assets/textures/blackstone.png",
        "assets/textures/smooth_quartz.png",
    ]
    .into_iter()
    .map(Texture::load)
    .collect()
}

fn build_scene() -> Vec<Cube> {
    vec![
        Cube::from_center_size(
            Vec3::new(-1.75, -0.35, -0.15),
            1.15,
            Material::textured(STONE),
        ),
        Cube::from_center_size(
            Vec3::new(-0.80, -0.10, -0.90),
            1.20,
            Material::textured(MOSS_BLOCK),
        ),
        Cube::from_center_size(
            Vec3::new(0.10, -0.35, 0.15),
            1.30,
            Material::textured(OBSIDIAN),
        ),
        Cube::from_center_size(
            Vec3::new(1.05, -0.05, -0.75),
            1.10,
            Material::textured(BLACKSTONE),
        ),
        Cube::from_center_size(
            Vec3::new(1.85, -0.40, 0.20),
            1.00,
            Material::textured(SMOOTH_QUARTZ),
        ),
    ]
}
