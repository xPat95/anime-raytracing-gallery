mod block_geometry;
mod bvh;
mod camera;
mod cube;
mod framebuffer;
mod gallery;
mod intersect;
mod light;
mod material;
mod materials;
mod ray_intersect;
mod render;
mod scene;
mod texture;
mod ui;
mod vec3;

use std::{env, time::Instant};

use bvh::Bvh;
use camera::Camera;
use framebuffer::Framebuffer;
use gallery::SceneConfig;
use light::LightingConfig;
use materials::MaterialCatalog;
use raylib::prelude::*;
use render::RenderStrategy;
use texture::Texture;
use ui::{AppState, GalleryLayout, GalleryState, UiAssets};

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

struct AppOptions {
    mode: RunMode,
    initial_scene_id: Option<String>,
}

struct ActiveScene {
    scene: scene::Scene,
    bvh: Bvh,
    camera: Camera,
    lighting: LightingConfig,
    textures: Vec<Texture>,
    framebuffer: Framebuffer,
    display_texture: Texture2D,
    pending_camera_render: bool,
}

impl ActiveScene {
    fn load(
        config: SceneConfig,
        mode: RunMode,
        raylib: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<Self, String> {
        println!("Selected scene: {} ({})", config.display_name, config.id);
        let camera = Camera::new(config.camera);
        let lighting = config.lighting;
        let (materials, textures) = MaterialCatalog::load(&config.textures, &config.materials)?;
        let (scene, bvh, framebuffer) =
            load_and_render_scene(&config, mode, &camera, &lighting, &materials, &textures)?;
        let image = framebuffer.to_image();
        let display_texture = raylib
            .load_texture_from_image(thread, &image)
            .map_err(|error| format!("could not create framebuffer texture: {error}"))?;
        display_texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_POINT);
        print_camera_state(&camera);
        Ok(Self {
            scene,
            bvh,
            camera,
            lighting,
            textures,
            framebuffer,
            display_texture,
            pending_camera_render: false,
        })
    }

    fn update_input(
        &mut self,
        raylib: &mut RaylibHandle,
        thread: &RaylibThread,
        ui_consumes_mouse: bool,
    ) -> Result<(), String> {
        let left_down = raylib.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
        let right_down = raylib.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);
        let mouse_delta = raylib.get_mouse_delta();
        if left_down && !ui_consumes_mouse {
            self.pending_camera_render |= self.camera.orbit(mouse_delta.x, mouse_delta.y);
        }
        if right_down {
            self.pending_camera_render |= self.camera.pan(mouse_delta.x, mouse_delta.y);
        }

        let orbit_x = key_axis(raylib, KeyboardKey::KEY_A, KeyboardKey::KEY_D);
        let orbit_y = key_axis(raylib, KeyboardKey::KEY_S, KeyboardKey::KEY_W);
        let pan_x = key_axis(raylib, KeyboardKey::KEY_LEFT, KeyboardKey::KEY_RIGHT);
        let pan_y = key_axis(raylib, KeyboardKey::KEY_DOWN, KeyboardKey::KEY_UP);
        let frame_time = raylib.get_frame_time();
        self.pending_camera_render |= self.camera.orbit_keyboard(orbit_x, orbit_y, frame_time);
        self.pending_camera_render |= self.camera.pan_keyboard(pan_x, pan_y, frame_time);
        self.pending_camera_render |= self.camera.zoom(raylib.get_mouse_wheel_move());
        if raylib.is_key_pressed(KeyboardKey::KEY_R) {
            self.pending_camera_render |= self.camera.reset();
        }

        let continuous = (left_down && !ui_consumes_mouse)
            || right_down
            || orbit_x != 0.0
            || orbit_y != 0.0
            || pan_x != 0.0
            || pan_y != 0.0;
        if self.pending_camera_render && !continuous {
            print_camera_state(&self.camera);
            self.framebuffer = render_scene(
                &self.camera,
                &self.scene,
                &self.bvh,
                &self.textures,
                &self.lighting,
                RenderStrategy::BvhMultiThread,
                "updated camera",
            )
            .0;
            let image = self.framebuffer.to_image();
            let texture = raylib
                .load_texture_from_image(thread, &image)
                .map_err(|error| format!("could not update framebuffer texture: {error}"))?;
            texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_POINT);
            self.display_texture = texture;
            self.pending_camera_render = false;
        }
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let options = parse_options()?;
    let (mut raylib, thread) = raylib::init()
        .size(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
        .title("Anime Ray Tracing Gallery")
        .resizable()
        .build();
    raylib.maximize_window();
    raylib.set_target_fps(60);

    let ui_assets = UiAssets::load(&mut raylib, &thread)?;
    let mut gallery_state = GalleryState::new();
    let mut app_state = AppState::Cover;
    let mut active_scene = None;
    if let Some(scene_id) = options.initial_scene_id.as_deref() {
        active_scene = Some(ActiveScene::load(
            find_scene(scene_id)?,
            options.mode,
            &mut raylib,
            &thread,
        )?);
        app_state = AppState::Scene;
    }

    let mut cover_bounds = Rectangle::default();
    let mut gallery_layout = GalleryLayout::default();
    println!("Scene controls: LMB/WASD orbit, RMB/arrows pan, wheel zoom, R reset");

    while !raylib.window_should_close() {
        let mouse = raylib.get_mouse_position();
        let left_pressed = raylib.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        match app_state {
            AppState::Cover => {
                if left_pressed && ui::contains(cover_bounds, mouse) {
                    app_state = AppState::Gallery;
                }
            }
            AppState::Gallery => {
                if raylib.is_key_pressed(KeyboardKey::KEY_LEFT)
                    || (left_pressed && ui::contains(gallery_layout.previous_button, mouse))
                {
                    gallery_state.previous();
                }
                if raylib.is_key_pressed(KeyboardKey::KEY_RIGHT)
                    || (left_pressed && ui::contains(gallery_layout.next_button, mouse))
                {
                    gallery_state.next();
                }
                if left_pressed && ui::contains(gallery_layout.preview_button, mouse) {
                    active_scene = Some(ActiveScene::load(
                        find_scene(gallery_state.current_entry().scene_id)?,
                        RunMode::Optimized,
                        &mut raylib,
                        &thread,
                    )?);
                    app_state = AppState::Scene;
                }
            }
            AppState::Scene => {
                let button = ui::book_button(raylib.get_screen_width(), raylib.get_screen_height());
                let consumes_mouse = ui::contains(button, mouse)
                    && raylib.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
                if left_pressed && ui::contains(button, mouse) {
                    active_scene = None;
                    app_state = AppState::Gallery;
                } else if let Some(scene) = active_scene.as_mut() {
                    scene.update_input(&mut raylib, &thread, consumes_mouse)?;
                }
            }
        }

        let selected_name = gallery::available_scenes()[gallery_state.current_page()].display_name;
        let screen_width = raylib.get_screen_width();
        let screen_height = raylib.get_screen_height();
        let mut drawing = raylib.begin_drawing(&thread);
        match app_state {
            AppState::Cover => cover_bounds = ui::draw_cover(&mut drawing),
            AppState::Gallery => {
                gallery_layout =
                    ui::draw_gallery(&mut drawing, &ui_assets, &gallery_state, selected_name);
            }
            AppState::Scene => {
                drawing.clear_background(Color::BLACK);
                if let Some(scene) = active_scene.as_ref() {
                    drawing.draw_texture_pro(
                        &scene.display_texture,
                        Rectangle::new(0.0, 0.0, RENDER_WIDTH as f32, RENDER_HEIGHT as f32),
                        fit_render_to_window(screen_width, screen_height),
                        Vector2::zero(),
                        0.0,
                        Color::WHITE,
                    );
                }
                ui::draw_book_button(&mut drawing, ui::book_button(screen_width, screen_height));
            }
        }
    }
    Ok(())
}

fn find_scene(scene_id: &str) -> Result<SceneConfig, String> {
    gallery::available_scenes()
        .into_iter()
        .find(|config| config.id == scene_id)
        .ok_or_else(|| format!("unknown scene '{scene_id}'"))
}

fn load_and_render_scene(
    config: &SceneConfig,
    mode: RunMode,
    camera: &Camera,
    lighting: &LightingConfig,
    materials: &MaterialCatalog,
    textures: &[Texture],
) -> Result<(scene::Scene, Bvh, Framebuffer), String> {
    if matches!(mode, RunMode::Compare) {
        let full = scene::load(config.scene_path, |name| materials.for_block(name), false)?;
        let optimized = scene::load(config.scene_path, |name| materials.for_block(name), true)?;
        print_scene_metrics(&optimized);
        let full_bvh = build_bvh(&full.primitives);
        let optimized_bvh = build_bvh(&optimized.primitives);
        print_bvh_metrics(&optimized_bvh);
        let (full_frame, full_time, _) = render_scene(
            camera,
            &full,
            &full_bvh,
            textures,
            lighting,
            RenderStrategy::BvhMultiThread,
            "full scene",
        );
        let (frame, optimized_time, _) = render_scene(
            camera,
            &optimized,
            &optimized_bvh,
            textures,
            lighting,
            RenderStrategy::BvhMultiThread,
            "optimized scene",
        );
        print_framebuffer_comparison("full vs optimized", &full_frame, &frame)?;
        println!("Render times: full {full_time:.2?}, optimized {optimized_time:.2?}");
        return Ok((optimized, optimized_bvh, frame));
    }

    let scene = scene::load(
        config.scene_path,
        |name| materials.for_block(name),
        !matches!(mode, RunMode::Full),
    )?;
    print_scene_metrics(&scene);
    let bvh = build_bvh(&scene.primitives);
    print_bvh_metrics(&bvh);
    if matches!(mode, RunMode::BenchmarkAccelerators) {
        let (linear, linear_time, _) = render_scene(
            camera,
            &scene,
            &bvh,
            textures,
            lighting,
            RenderStrategy::LinearSingleThread,
            "linear single-thread",
        );
        let (single, single_time, _) = render_scene(
            camera,
            &scene,
            &bvh,
            textures,
            lighting,
            RenderStrategy::BvhSingleThread,
            "BVH single-thread",
        );
        print_framebuffer_comparison("linear vs BVH", &linear, &single)?;
        let (frame, multi_time, workers) = render_scene(
            camera,
            &scene,
            &bvh,
            textures,
            lighting,
            RenderStrategy::BvhMultiThread,
            "BVH multi-thread",
        );
        print_framebuffer_comparison("BVH single vs multi", &single, &frame)?;
        println!(
            "Benchmark: linear {linear_time:.2?}, BVH {single_time:.2?}, multi {multi_time:.2?}, {workers} workers"
        );
        return Ok((scene, bvh, frame));
    }
    let label = if matches!(mode, RunMode::Full) {
        "full scene"
    } else {
        "optimized scene"
    };
    let frame = render_scene(
        camera,
        &scene,
        &bvh,
        textures,
        lighting,
        RenderStrategy::BvhMultiThread,
        label,
    )
    .0;
    Ok((scene, bvh, frame))
}

fn print_camera_state(camera: &Camera) {
    println!(
        "Camera: yaw {:.2} deg, pitch {:.2} deg, distance {:.2}",
        camera.yaw_degrees(),
        camera.pitch_degrees(),
        camera.radius()
    );
}

fn key_axis(raylib: &RaylibHandle, negative: KeyboardKey, positive: KeyboardKey) -> f32 {
    raylib.is_key_down(positive) as u8 as f32 - raylib.is_key_down(negative) as u8 as f32
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

fn parse_options() -> Result<AppOptions, String> {
    let mut mode = RunMode::Optimized;
    let mut initial_scene_id = None;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--scene" => {
                initial_scene_id = Some(arguments.next().ok_or("--scene requires a scene id")?)
            }
            "--no-culling" => mode = RunMode::Full,
            "--compare-culling" => mode = RunMode::Compare,
            "--benchmark-accelerators" => mode = RunMode::BenchmarkAccelerators,
            _ => return Err(format!("unknown argument '{argument}'")),
        }
    }
    if initial_scene_id.is_none() && !matches!(mode, RunMode::Optimized) {
        initial_scene_id = Some("black_clover_skull".to_owned());
    }
    Ok(AppOptions {
        mode,
        initial_scene_id,
    })
}

fn build_bvh(primitives: &[crate::cube::Cube]) -> Bvh {
    let started = Instant::now();
    let bvh = Bvh::build(primitives);
    println!("BVH built in {:.2?}", started.elapsed());
    bvh
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
        "Full cube blocks: {} original, {} opaque enclosed, {} retained",
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
