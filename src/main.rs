mod camera;
mod cube;
mod framebuffer;
mod intersect;
mod material;
mod ray_intersect;
mod render;
mod scene;
mod texture;
mod vec3;

use std::time::Instant;

use camera::Camera;
use framebuffer::Framebuffer;
use material::Material;
use raylib::prelude::*;
use texture::Texture;
use vec3::Vec3;

const RENDER_WIDTH: u32 = 240;
const RENDER_HEIGHT: u32 = 135;
const WINDOW_SCALE: u32 = 4;

const STONE: usize = 0;
const MOSS_BLOCK: usize = 1;
const OBSIDIAN: usize = 2;
const SMOOTH_QUARTZ: usize = 3;
const NETHER_PORTAL: usize = 4;

fn main() -> Result<(), String> {
    let mut framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT, Color::BLACK);
    let camera = Camera::look_at(
        Vec3::new(145.0, 95.0, 180.0),
        Vec3::new(0.0, 0.0, 0.0),
        48.0,
    );
    let textures = load_textures()?;
    let scene = scene::load("assets/scenes/proyecto.scene", material_for_block)?;

    println!(
        "Loaded {} cubes from a {} x {} x {} scene (offset: {}, {}, {})",
        scene.cubes.len(),
        scene.dimensions[0],
        scene.dimensions[1],
        scene.dimensions[2],
        scene.offset.x,
        scene.offset.y,
        scene.offset.z
    );
    println!(
        "Temporarily omitted: {} slabs, {} stairs, {} walls",
        scene.omitted_slabs, scene.omitted_stairs, scene.omitted_walls
    );
    println!("Rendering {RENDER_WIDTH} x {RENDER_HEIGHT} primary rays...");
    let render_started = Instant::now();
    render::render(&mut framebuffer, &camera, &scene.cubes, &textures);
    println!("Render completed in {:.2?}", render_started.elapsed());

    let (mut raylib, thread) = raylib::init()
        .size(
            (RENDER_WIDTH * WINDOW_SCALE) as i32,
            (RENDER_HEIGHT * WINDOW_SCALE) as i32,
        )
        .title("Black Clover Ray Tracing Skull - Imported Schematic")
        .build();

    raylib.set_target_fps(60);

    let image = framebuffer.to_image();
    let texture = raylib
        .load_texture_from_image(&thread, &image)
        .map_err(|error| format!("could not create texture from framebuffer: {error}"))?;

    while !raylib.window_should_close() {
        let mut drawing = raylib.begin_drawing(&thread);
        drawing.clear_background(Color::BLACK);
        drawing.draw_texture_ex(
            &texture,
            Vector2::new(0.0, 0.0),
            0.0,
            WINDOW_SCALE as f32,
            Color::WHITE,
        );
    }

    Ok(())
}

fn load_textures() -> Result<Vec<Texture>, String> {
    Ok(vec![
        Texture::load("assets/textures/stone.png")?,
        Texture::load("assets/textures/moss_block.png")?,
        Texture::load("assets/textures/obsidian.png")?,
        Texture::load("assets/textures/smooth_quartz.png")?,
        Texture::load_first_frame("assets/textures/nether_portal.png", 16)?,
    ])
}

fn material_for_block(block_name: &str) -> Result<Option<Material>, String> {
    let texture_index = match block_name {
        "minecraft:air" => return Ok(None),
        "minecraft:stone" => STONE,
        "minecraft:moss_block" => MOSS_BLOCK,
        "minecraft:obsidian" => OBSIDIAN,
        "minecraft:smooth_quartz" => SMOOTH_QUARTZ,
        "minecraft:nether_portal" => NETHER_PORTAL,
        _ => return Err(format!("scene contains unsupported block '{block_name}'")),
    };

    Ok(Some(Material::textured(texture_index)))
}
