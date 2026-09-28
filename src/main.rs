mod block_geometry;
mod camera;
mod cube;
mod framebuffer;
mod intersect;
mod light;
mod material;
mod materials;
mod ray_intersect;
mod render;
mod scene;
mod texture;
mod vec3;

use std::{env, time::Instant};

use camera::Camera;
use framebuffer::Framebuffer;
use light::LightingConfig;
use materials::MaterialCatalog;
use raylib::prelude::*;
use texture::Texture;
use vec3::Vec3;

const RENDER_WIDTH: u32 = 480;
const RENDER_HEIGHT: u32 = 270;
const INITIAL_WINDOW_WIDTH: i32 = 1280;
const INITIAL_WINDOW_HEIGHT: i32 = 720;

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
        43.0,
    );
    let lighting = LightingConfig::scene();
    let (materials, textures) = MaterialCatalog::load()?;
    let framebuffer = match mode {
        RunMode::Compare => {
            let full_scene = scene::load(
                "assets/scenes/proyecto.scene",
                |block_name| materials.for_block(block_name),
                false,
            )?;
            let optimized_scene = scene::load(
                "assets/scenes/proyecto.scene",
                |block_name| materials.for_block(block_name),
                true,
            )?;
            print_scene_metrics(&optimized_scene);

            let (full_framebuffer, full_time) =
                render_scene(&camera, &full_scene, &textures, &lighting, "full scene");
            let (optimized_framebuffer, optimized_time) = render_scene(
                &camera,
                &optimized_scene,
                &textures,
                &lighting,
                "optimized scene",
            );
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
                |block_name| materials.for_block(block_name),
                culling_enabled,
            )?;
            print_scene_metrics(&scene);
            let label = if culling_enabled {
                "optimized scene"
            } else {
                "full scene"
            };
            render_scene(&camera, &scene, &textures, &lighting, label).0
        }
    };

    let (mut raylib, thread) = raylib::init()
        .size(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
        .title("Black Clover Ray Tracing Skull - Imported Schematic")
        .resizable()
        .build();

    raylib.maximize_window();
    raylib.set_target_fps(60);

    let image = framebuffer.to_image();
    let texture = raylib
        .load_texture_from_image(&thread, &image)
        .map_err(|error| format!("could not create texture from framebuffer: {error}"))?;
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);

    while !raylib.window_should_close() {
        let destination =
            fit_render_to_window(raylib.get_screen_width(), raylib.get_screen_height());
        let mut drawing = raylib.begin_drawing(&thread);
        drawing.clear_background(Color::BLACK);
        drawing.draw_texture_pro(
            &texture,
            Rectangle::new(0.0, 0.0, RENDER_WIDTH as f32, RENDER_HEIGHT as f32),
            destination,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
    }

    Ok(())
}

fn fit_render_to_window(window_width: i32, window_height: i32) -> Rectangle {
    let scale = (window_width as f32 / RENDER_WIDTH as f32)
        .min(window_height as f32 / RENDER_HEIGHT as f32);
    let width = RENDER_WIDTH as f32 * scale;
    let height = RENDER_HEIGHT as f32 * scale;

    Rectangle::new(
        (window_width as f32 - width) * 0.5,
        (window_height as f32 - height) * 0.5,
        width,
        height,
    )
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
    lighting: &LightingConfig,
    label: &str,
) -> (Framebuffer, std::time::Duration) {
    let mut framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT, Color::BLACK);
    println!("Rendering {label} at {RENDER_WIDTH} x {RENDER_HEIGHT}...");
    let started = Instant::now();
    render::render(
        &mut framebuffer,
        camera,
        &scene.primitives,
        textures,
        lighting,
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_rectangle_preserves_aspect_ratio_in_wide_window() {
        let rectangle = fit_render_to_window(1600, 800);

        assert!(((rectangle.width / rectangle.height) - (16.0 / 9.0)).abs() < 0.00001);
        assert!(rectangle.x > 0.0);
        assert_eq!(rectangle.y, 0.0);
    }

    #[test]
    fn render_rectangle_preserves_aspect_ratio_in_tall_window() {
        let rectangle = fit_render_to_window(800, 1000);

        assert!(((rectangle.width / rectangle.height) - (16.0 / 9.0)).abs() < 0.00001);
        assert_eq!(rectangle.x, 0.0);
        assert!(rectangle.y > 0.0);
    }
}
