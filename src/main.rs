mod block_geometry;
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

use std::{env, time::Instant};

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
const BLACKSTONE: usize = 5;

#[derive(Clone, Copy)]
enum RunMode {
    Optimized,
    Full,
    Compare,
}

fn main() -> Result<(), String> {
    let mode = parse_run_mode()?;
    let camera = Camera::look_at(
        Vec3::new(145.0, 95.0, 180.0),
        Vec3::new(0.0, 0.0, 0.0),
        48.0,
    );
    let textures = load_textures()?;
    let framebuffer = match mode {
        RunMode::Compare => {
            let full_scene =
                scene::load("assets/scenes/proyecto.scene", material_for_block, false)?;
            let optimized_scene =
                scene::load("assets/scenes/proyecto.scene", material_for_block, true)?;
            print_scene_metrics(&optimized_scene);

            let (full_framebuffer, full_time) =
                render_scene(&camera, &full_scene, &textures, "full scene");
            let (optimized_framebuffer, optimized_time) =
                render_scene(&camera, &optimized_scene, &textures, "optimized scene");
            let difference = full_framebuffer.difference(&optimized_framebuffer)?;
            println!(
                "Framebuffer comparison: {} different pixels, maximum channel difference {}",
                difference.different_pixels, difference.maximum_channel_difference
            );
            println!(
                "Render times: full {:.2?}, optimized {:.2?}",
                full_time, optimized_time
            );
            optimized_framebuffer
        }
        RunMode::Optimized | RunMode::Full => {
            let culling_enabled = matches!(mode, RunMode::Optimized);
            let scene = scene::load(
                "assets/scenes/proyecto.scene",
                material_for_block,
                culling_enabled,
            )?;
            print_scene_metrics(&scene);
            let label = if culling_enabled {
                "optimized scene"
            } else {
                "full scene"
            };
            render_scene(&camera, &scene, &textures, label).0
        }
    };

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

fn parse_run_mode() -> Result<RunMode, String> {
    match env::args().nth(1).as_deref() {
        None => Ok(RunMode::Optimized),
        Some("--no-culling") => Ok(RunMode::Full),
        Some("--compare-culling") => Ok(RunMode::Compare),
        Some(argument) => Err(format!(
            "unknown argument '{argument}'; use --no-culling or --compare-culling"
        )),
    }
}

fn render_scene(
    camera: &Camera,
    scene: &scene::Scene,
    textures: &[Texture],
    label: &str,
) -> (Framebuffer, std::time::Duration) {
    let mut framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT, Color::BLACK);
    println!("Rendering {label} at {RENDER_WIDTH} x {RENDER_HEIGHT}...");
    let started = Instant::now();
    render::render(&mut framebuffer, camera, &scene.primitives, textures);
    let elapsed = started.elapsed();
    println!("{label} completed in {elapsed:.2?}");
    (framebuffer, elapsed)
}

fn print_scene_metrics(scene: &scene::Scene) {
    let before = scene.primitives_before_culling();
    let after = scene.primitives.len();
    let reduction = (before - after) as f64 / before as f64 * 100.0;

    println!(
        "Loaded {} Minecraft blocks from a {} x {} x {} scene",
        scene.block_count(),
        scene.dimensions[0],
        scene.dimensions[1],
        scene.dimensions[2]
    );
    println!(
        "Opaque full blocks: {} original, {} enclosed, {} retained",
        scene.full_blocks, scene.enclosed_full_blocks, scene.retained_full_blocks
    );
    println!(
        "Always retained: {} portals, {} slabs, {} stairs, {} walls",
        scene.portal_blocks, scene.slabs, scene.stairs, scene.walls
    );
    println!(
        "AABB primitives: {before} before, {after} after ({reduction:.2}% reduction); offset: {}, {}, {}",
        scene.offset.x, scene.offset.y, scene.offset.z
    );
}

fn load_textures() -> Result<Vec<Texture>, String> {
    Ok(vec![
        Texture::load("assets/textures/stone.png")?,
        Texture::load("assets/textures/moss_block.png")?,
        Texture::load("assets/textures/obsidian.png")?,
        Texture::load("assets/textures/smooth_quartz.png")?,
        Texture::load_first_frame("assets/textures/nether_portal.png", 16)?,
        Texture::load("assets/textures/blackstone.png")?,
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
        "minecraft:blackstone_slab"
        | "minecraft:blackstone_stairs"
        | "minecraft:blackstone_wall" => BLACKSTONE,
        _ => return Err(format!("scene contains unsupported block '{block_name}'")),
    };

    Ok(Some(Material::textured(texture_index)))
}
