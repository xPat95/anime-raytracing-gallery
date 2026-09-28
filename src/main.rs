mod block_geometry;
mod bvh;
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

use bvh::Bvh;
use camera::Camera;
use framebuffer::Framebuffer;
use light::LightingConfig;
use materials::MaterialCatalog;
use raylib::prelude::*;
use render::RenderStrategy;
use texture::Texture;

const RENDER_WIDTH: u32 = 480;
const RENDER_HEIGHT: u32 = 270;
const INITIAL_WINDOW_WIDTH: i32 = 1280;
const INITIAL_WINDOW_HEIGHT: i32 = 720;

#[derive(Clone, Copy)]
enum RunMode {
    Optimized,
    Full,
    Compare,
    BenchmarkAccelerators,
}

fn main() -> Result<(), String> {
    let mode = parse_run_mode()?;
    let mut camera = Camera::scene();
    let lighting = LightingConfig::scene();
    let (materials, textures) = MaterialCatalog::load()?;
    let (scene, bvh, mut framebuffer) = match mode {
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
            let full_bvh = Bvh::build(&full_scene.primitives);
            let optimized_bvh = Bvh::build(&optimized_scene.primitives);
            print_bvh_metrics(&optimized_bvh);

            let (full_framebuffer, full_time, _) = render_scene(
                &camera,
                &full_scene,
                &full_bvh,
                &textures,
                &lighting,
                RenderStrategy::BvhMultiThread,
                "full scene",
            );
            let (optimized_framebuffer, optimized_time, _) = render_scene(
                &camera,
                &optimized_scene,
                &optimized_bvh,
                &textures,
                &lighting,
                RenderStrategy::BvhMultiThread,
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
            (optimized_scene, optimized_bvh, optimized_framebuffer)
        }
        RunMode::BenchmarkAccelerators => {
            let scene = scene::load(
                "assets/scenes/proyecto.scene",
                |block_name| materials.for_block(block_name),
                true,
            )?;
            print_scene_metrics(&scene);
            let bvh = Bvh::build(&scene.primitives);
            print_bvh_metrics(&bvh);
            let (linear, linear_time, _) = render_scene(
                &camera,
                &scene,
                &bvh,
                &textures,
                &lighting,
                RenderStrategy::LinearSingleThread,
                "linear single-thread",
            );
            let (bvh_single, bvh_time, _) = render_scene(
                &camera,
                &scene,
                &bvh,
                &textures,
                &lighting,
                RenderStrategy::BvhSingleThread,
                "BVH single-thread",
            );
            print_framebuffer_comparison("linear vs BVH", &linear, &bvh_single)?;
            let (bvh_multi, multi_time, workers) = render_scene(
                &camera,
                &scene,
                &bvh,
                &textures,
                &lighting,
                RenderStrategy::BvhMultiThread,
                "BVH multi-thread",
            );
            print_framebuffer_comparison("BVH single vs multi", &bvh_single, &bvh_multi)?;
            println!(
                "Benchmark: linear {:.2?}, BVH {:.2?} ({:.2}x), BVH multi {:.2?} ({:.2}x total, {:.2}x vs BVH, {workers} workers)",
                linear_time,
                bvh_time,
                linear_time.as_secs_f64() / bvh_time.as_secs_f64(),
                multi_time,
                linear_time.as_secs_f64() / multi_time.as_secs_f64(),
                bvh_time.as_secs_f64() / multi_time.as_secs_f64(),
            );
            (scene, bvh, bvh_multi)
        }
        RunMode::Optimized | RunMode::Full => {
            let culling_enabled = matches!(mode, RunMode::Optimized);
            let scene = scene::load(
                "assets/scenes/proyecto.scene",
                |block_name| materials.for_block(block_name),
                culling_enabled,
            )?;
            print_scene_metrics(&scene);
            let bvh = Bvh::build(&scene.primitives);
            print_bvh_metrics(&bvh);
            let label = if culling_enabled {
                "optimized scene"
            } else {
                "full scene"
            };
            let framebuffer = render_scene(
                &camera,
                &scene,
                &bvh,
                &textures,
                &lighting,
                RenderStrategy::BvhMultiThread,
                label,
            )
            .0;
            (scene, bvh, framebuffer)
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
    let mut texture = raylib
        .load_texture_from_image(&thread, &image)
        .map_err(|error| format!("could not create texture from framebuffer: {error}"))?;
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);
    let mut pending_camera_render = false;

    println!("Controls: left mouse drag = orbit, wheel = zoom, R = reset");
    print_camera_state(&camera);

    while !raylib.window_should_close() {
        if raylib.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let delta = raylib.get_mouse_delta();
            pending_camera_render |= camera.orbit(delta.x, delta.y);
        }

        pending_camera_render |= camera.zoom(raylib.get_mouse_wheel_move());
        if raylib.is_key_pressed(KeyboardKey::KEY_R) {
            pending_camera_render |= camera.reset();
        }

        if pending_camera_render && !raylib.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            print_camera_state(&camera);
            framebuffer = render_scene(
                &camera,
                &scene,
                &bvh,
                &textures,
                &lighting,
                RenderStrategy::BvhMultiThread,
                "updated camera",
            )
            .0;
            let image = framebuffer.to_image();
            let new_texture = raylib
                .load_texture_from_image(&thread, &image)
                .map_err(|error| format!("could not update framebuffer texture: {error}"))?;
            new_texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);
            texture = new_texture;
            pending_camera_render = false;
        }

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

fn print_camera_state(camera: &Camera) {
    println!(
        "Camera: yaw {:.2}°, pitch {:.2}°, distance {:.2}",
        camera.yaw_degrees(),
        camera.pitch_degrees(),
        camera.radius()
    );
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
        Some("--benchmark-accelerators") => Ok(RunMode::BenchmarkAccelerators),
        Some(argument) => Err(format!(
            "unknown argument '{argument}'; use --no-culling, --compare-culling, or --benchmark-accelerators"
        )),
    }
}

fn render_scene(
    camera: &Camera,
    scene: &scene::Scene,
    bvh: &Bvh,
    textures: &[Texture],
    lighting: &LightingConfig,
    strategy: RenderStrategy,
    label: &str,
) -> (Framebuffer, std::time::Duration, usize) {
    let mut framebuffer = Framebuffer::new(RENDER_WIDTH, RENDER_HEIGHT, Color::BLACK);
    println!("Rendering {label} at {RENDER_WIDTH} x {RENDER_HEIGHT}...");
    let started = Instant::now();
    let workers = render::render(
        &mut framebuffer,
        camera,
        &scene.primitives,
        bvh,
        textures,
        lighting,
        strategy,
    );
    let elapsed = started.elapsed();
    println!("{label} completed in {elapsed:.2?}");
    (framebuffer, elapsed, workers)
}

fn print_framebuffer_comparison(
    label: &str,
    left: &Framebuffer,
    right: &Framebuffer,
) -> Result<(), String> {
    let difference = left.difference(right)?;
    println!(
        "{label}: {} different pixels, maximum channel difference {}",
        difference.different_pixels, difference.maximum_channel_difference
    );
    Ok(())
}

fn print_bvh_metrics(bvh: &Bvh) {
    println!(
        "BVH: {} nodes, maximum depth {}, leaf size {}",
        bvh.node_count(),
        bvh.max_depth(),
        bvh::LEAF_SIZE
    );
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
