mod audio;
mod block_geometry;
mod bvh;
mod camera;
mod cube;
mod framebuffer;
mod gallery;
mod ground;
mod intersect;
mod light;
mod material;
mod materials;
mod menu_background;
mod ray_intersect;
mod render;
mod scene;
mod sky;
mod texture;
mod ui;
mod vec3;

use std::{
    env,
    time::{Duration, Instant},
};

use audio::AudioManager;
use bvh::Bvh;
use camera::Camera;
use framebuffer::Framebuffer;
use gallery::SceneConfig;
use ground::GroundPlane;
use light::LightingConfig;
use materials::MaterialCatalog;
use raylib::audio::RaylibAudio;
use raylib::prelude::*;
use render::RenderStrategy;
use sky::SkyType;
use texture::Texture;
use ui::{
    AppState, AudioUiState, GalleryLayout, GalleryState, UiAssets, UiTransition, UiTransitionKind,
};

const INITIAL_WINDOW_WIDTH: i32 = 1280;
const INITIAL_WINDOW_HEIGHT: i32 = 720;
const RENDER_ASPECT_WIDTH: u32 = 16;
const RENDER_ASPECT_HEIGHT: u32 = 9;

const RENDER_QUALITY: RenderQuality = RenderQuality {
    interactive_max_width: 640,
    interactive_max_height: 360,
    final_max_width: 1280,
    final_max_height: 720,
    settle_delay: Duration::from_millis(200),
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RenderResolution {
    width: u32,
    height: u32,
}

impl RenderResolution {
    fn pixel_count(self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

#[derive(Clone, Copy)]
struct RenderQuality {
    interactive_max_width: u32,
    interactive_max_height: u32,
    final_max_width: u32,
    final_max_height: u32,
    settle_delay: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QualityLevel {
    Interactive,
    Final,
}

#[derive(Clone, Copy)]
enum RunMode {
    Optimized,
    Full,
    Compare,
    BenchmarkAccelerators,
    BenchmarkQuality,
}

struct AppOptions {
    mode: RunMode,
    initial_scene_id: Option<String>,
    generate_menu_backgrounds: Option<bool>,
    teaser_camera_scene_id: Option<String>,
}

struct ActiveScene {
    scene: scene::Scene,
    bvh: Bvh,
    camera: Camera,
    lighting: LightingConfig,
    sky: SkyType,
    ground: GroundPlane,
    textures: Vec<Texture>,
    framebuffer: Framebuffer,
    display_texture: Texture2D,
    rendered_resolution: RenderResolution,
    quality_level: QualityLevel,
    last_interaction: Instant,
    animation_started: Instant,
    rendered_animation_frames: Vec<usize>,
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
        let sky = config.sky;
        let (materials, textures) = MaterialCatalog::load(&config.textures, &config.materials)?;
        let rendered_animation_frames = animation_frames(&textures, 0.0);
        let ground = GroundPlane::from_config(config.ground)?;
        let resolution = target_resolution(
            raylib.get_screen_width(),
            raylib.get_screen_height(),
            QualityLevel::Final,
        );
        let (scene, bvh, framebuffer) = load_and_render_scene(
            &config, mode, &camera, &lighting, &materials, &textures, &ground, resolution,
        )?;
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
            sky,
            ground,
            textures,
            framebuffer,
            display_texture,
            rendered_resolution: resolution,
            quality_level: QualityLevel::Final,
            last_interaction: Instant::now(),
            animation_started: Instant::now(),
            rendered_animation_frames,
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
        let mut camera_changed = false;
        if left_down && !ui_consumes_mouse {
            camera_changed |= self.camera.orbit(mouse_delta.x, mouse_delta.y);
        }
        if right_down {
            camera_changed |= self.camera.pan(mouse_delta.x, mouse_delta.y);
        }

        let orbit_x = key_axis(raylib, KeyboardKey::KEY_A, KeyboardKey::KEY_D);
        let orbit_y = key_axis(raylib, KeyboardKey::KEY_S, KeyboardKey::KEY_W);
        let pan_x = key_axis(raylib, KeyboardKey::KEY_LEFT, KeyboardKey::KEY_RIGHT);
        let pan_y = key_axis(raylib, KeyboardKey::KEY_DOWN, KeyboardKey::KEY_UP);
        let frame_time = raylib.get_frame_time();
        camera_changed |= self.camera.orbit_keyboard(orbit_x, orbit_y, frame_time);
        camera_changed |= self.camera.pan_keyboard(pan_x, pan_y, frame_time);
        camera_changed |= self.camera.zoom(raylib.get_mouse_wheel_move());
        if raylib.is_key_pressed(KeyboardKey::KEY_R) {
            camera_changed |= self.camera.reset();
        }

        let now = Instant::now();
        let desired_level = if camera_changed {
            self.last_interaction = now;
            QualityLevel::Interactive
        } else if let Some(settled) = settled_quality(now.duration_since(self.last_interaction)) {
            settled
        } else {
            self.quality_level
        };
        let desired_resolution = target_resolution(
            raylib.get_screen_width(),
            raylib.get_screen_height(),
            desired_level,
        );
        let render_time_seconds = now.duration_since(self.animation_started).as_secs_f32();
        let current_animation_frames = animation_frames(&self.textures, render_time_seconds);
        let animation_changed = current_animation_frames != self.rendered_animation_frames;
        let needs_render = camera_changed
            || desired_level != self.quality_level
            || desired_resolution != self.rendered_resolution
            || animation_changed;
        if needs_render {
            print_camera_state(&self.camera);
            let label = match desired_level {
                QualityLevel::Interactive => "interactive camera",
                QualityLevel::Final => "final camera",
            };
            self.framebuffer = render_scene_at(
                &self.camera,
                &self.scene,
                &self.bvh,
                &self.textures,
                &self.lighting,
                self.sky,
                &self.ground,
                render_time_seconds,
                RenderStrategy::BvhMultiThread,
                label,
                desired_resolution,
            )
            .0;
            let image = self.framebuffer.to_image();
            let texture = raylib
                .load_texture_from_image(thread, &image)
                .map_err(|error| format!("could not update framebuffer texture: {error}"))?;
            texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_POINT);
            self.display_texture = texture;
            self.rendered_resolution = desired_resolution;
            self.quality_level = desired_level;
            self.rendered_animation_frames = current_animation_frames;
        }
        Ok(())
    }
}

fn animation_frames(textures: &[Texture], elapsed_seconds: f32) -> Vec<usize> {
    textures
        .iter()
        .map(|texture| texture.frame_index(elapsed_seconds))
        .collect()
}

fn main() -> Result<(), String> {
    let options = parse_options()?;
    let (mut raylib, thread) = raylib::init()
        .size(INITIAL_WINDOW_WIDTH, INITIAL_WINDOW_HEIGHT)
        .title("Anime Ray Tracing Gallery")
        .resizable()
        .build();
    if let Some(samples_only) = options.generate_menu_backgrounds {
        return generate_menu_backgrounds(samples_only);
    }
    raylib.maximize_window();
    raylib.set_target_fps(60);

    let ui_assets = UiAssets::load(&mut raylib, &thread)?;
    let mut gallery_state = GalleryState::new();
    let mut menu_background = menu_background::MenuBackground::load(&mut raylib, &thread);
    let splash = menu_background::choose_splash();
    let menu_started = Instant::now();
    println!(
        "Menu background: {} frames loaded, approximately {:.1} MiB of RGBA texture data",
        menu_background.loaded_frames(),
        menu_background.approximate_gpu_bytes() as f32 / (1024.0 * 1024.0)
    );
    let mut app_state = AppState::MainMenu;
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

    // Start streamed audio only after blocking startup asset and scene work is complete.
    let audio_device = match RaylibAudio::init_audio_device() {
        Ok(audio) => {
            // Animated ray-traced frames can block the main loop for over 100 ms.
            // Give streamed music enough decoded audio to bridge those renders.
            audio.set_audio_stream_buffer_size_default(65_536);
            Some(audio)
        }
        Err(error) => {
            eprintln!("Audio device unavailable: {error}");
            None
        }
    };
    let initial_audio_scene = options
        .initial_scene_id
        .as_deref()
        .unwrap_or("black_clover_skull");
    let mut audio_manager = audio_device
        .as_ref()
        .map(|audio| AudioManager::load(audio, initial_audio_scene));

    let mut main_menu_button = Rectangle::default();
    let mut cover_layout = ui::CoverLayout::default();
    let mut gallery_layout = GalleryLayout::default();
    let mut transition: Option<UiTransition> = None;
    let mut transition_switched = false;
    println!("Scene controls: LMB/WASD orbit, RMB/arrows pan, wheel zoom, R reset");

    while !raylib.window_should_close() {
        let frame_time = raylib.get_frame_time();
        menu_background.update(frame_time);
        if let Some(audio) = audio_manager.as_mut() {
            audio.update(frame_time);
        }
        let mouse = raylib.get_mouse_position();
        let left_pressed = raylib.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        if let Some(active_transition) = transition.as_mut() {
            active_transition.update(frame_time);
            if active_transition.midpoint_reached() && !transition_switched {
                match active_transition.kind {
                    UiTransitionKind::PageTurnForward => {
                        gallery_state.next();
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.select_scene(gallery_state.current_entry().scene_id);
                        }
                    }
                    UiTransitionKind::PageTurnBackward => {
                        gallery_state.previous();
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.select_scene(gallery_state.current_entry().scene_id);
                        }
                    }
                    UiTransitionKind::FadeToScene => {
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.update(0.0);
                        }
                        active_scene = Some(ActiveScene::load(
                            find_scene(gallery_state.current_entry().scene_id)?,
                            RunMode::Optimized,
                            &mut raylib,
                            &thread,
                        )?);
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.update(0.0);
                        }
                        app_state = AppState::Scene;
                    }
                    UiTransitionKind::FadeToGallery => {
                        active_scene = None;
                        app_state = AppState::Gallery;
                    }
                    UiTransitionKind::OpeningBook | UiTransitionKind::ClosingBook => {}
                }
                transition_switched = true;
            }
            if active_transition.finished() {
                if active_transition.kind == UiTransitionKind::ClosingBook {
                    app_state = AppState::Cover;
                }
                transition = None;
                transition_switched = false;
            }
        }

        if transition.is_none() {
            match app_state {
                AppState::MainMenu => {
                    if left_pressed && ui::contains(main_menu_button, mouse) {
                        app_state = AppState::Cover;
                    }
                }
                AppState::Cover => {
                    if left_pressed && ui::contains(cover_layout.book, mouse) {
                        app_state = AppState::Gallery;
                        transition = Some(UiTransition::new(UiTransitionKind::OpeningBook));
                    } else if left_pressed && ui::contains(cover_layout.back_button, mouse) {
                        app_state = AppState::MainMenu;
                    }
                }
                AppState::Gallery => {
                    if gallery_state.can_previous()
                        && (raylib.is_key_pressed(KeyboardKey::KEY_LEFT)
                            || (left_pressed
                                && ui::contains(gallery_layout.previous_button, mouse)))
                    {
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.cancel_intro_and_resume_background();
                        }
                        transition = Some(UiTransition::new(UiTransitionKind::PageTurnBackward));
                    } else if gallery_state.can_next()
                        && (raylib.is_key_pressed(KeyboardKey::KEY_RIGHT)
                            || (left_pressed && ui::contains(gallery_layout.next_button, mouse)))
                    {
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.cancel_intro_and_resume_background();
                        }
                        transition = Some(UiTransition::new(UiTransitionKind::PageTurnForward));
                    } else if left_pressed && ui::contains(gallery_layout.close_button, mouse) {
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.cancel_intro_and_resume_background();
                        }
                        transition = Some(UiTransition::new(UiTransitionKind::ClosingBook));
                    }
                    if left_pressed && ui::contains(gallery_layout.audio_button, mouse) {
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.toggle_intro();
                        }
                    }
                    if left_pressed && ui::contains(gallery_layout.audio_progress, mouse) {
                        let progress = (mouse.x - gallery_layout.audio_progress.x)
                            / gallery_layout.audio_progress.width;
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.seek(progress);
                        }
                    }
                    if left_pressed && ui::contains(gallery_layout.preview_button, mouse) {
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.cancel_intro_and_resume_background();
                        }
                        transition = Some(UiTransition::new(UiTransitionKind::FadeToScene));
                    }
                }
                AppState::Scene => {
                    if raylib.is_key_pressed(KeyboardKey::KEY_P) {
                        if let (Some(scene_id), Some(scene)) = (
                            options.teaser_camera_scene_id.as_deref(),
                            active_scene.as_ref(),
                        ) {
                            print_teaser_camera_snapshot(scene_id, &scene.camera);
                        }
                    }
                    let button =
                        ui::book_button(raylib.get_screen_width(), raylib.get_screen_height());
                    let audio_button = ui::scene_audio_button(button);
                    let consumes_mouse = (ui::contains(button, mouse)
                        || ui::contains(audio_button, mouse))
                        && raylib.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
                    if left_pressed && ui::contains(button, mouse) {
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.cancel_intro_and_resume_background();
                        }
                        transition = Some(UiTransition::new(UiTransitionKind::FadeToGallery));
                    } else if left_pressed && ui::contains(audio_button, mouse) {
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.toggle_intro();
                        }
                    } else if let Some(scene) = active_scene.as_mut() {
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.update(0.0);
                        }
                        scene.update_input(&mut raylib, &thread, consumes_mouse)?;
                        if let Some(audio) = audio_manager.as_mut() {
                            audio.update(0.0);
                        }
                    }
                }
            }
        }

        let selected_name = gallery::available_scenes()[gallery_state.current_page()].display_name;
        let screen_width = raylib.get_screen_width();
        let screen_height = raylib.get_screen_height();
        let audio_status = audio_manager
            .as_ref()
            .map_or(AudioUiState::default(), |audio| {
                let status = audio.status();
                AudioUiState {
                    active: status.active,
                    playing: status.playing,
                    progress: status.progress,
                    elapsed: status.elapsed,
                    duration: status.duration,
                }
            });
        let mut drawing = raylib.begin_drawing(&thread);
        match app_state {
            AppState::MainMenu => {
                menu_background.draw(&mut drawing, 118);
                main_menu_button = ui::draw_main_menu(
                    &mut drawing,
                    splash,
                    menu_started.elapsed().as_secs_f32() * 2.2,
                    mouse,
                );
            }
            AppState::Cover => {
                menu_background.draw(&mut drawing, 138);
                cover_layout = ui::draw_book_cover(&mut drawing, mouse);
            }
            AppState::Gallery => {
                menu_background.draw(&mut drawing, 158);
                gallery_layout = ui::draw_gallery(
                    &mut drawing,
                    &ui_assets,
                    &gallery_state,
                    selected_name,
                    audio_status,
                    mouse,
                );
            }
            AppState::Scene => {
                drawing.clear_background(Color::BLACK);
                if let Some(scene) = active_scene.as_ref() {
                    drawing.draw_texture_pro(
                        &scene.display_texture,
                        Rectangle::new(
                            0.0,
                            0.0,
                            scene.rendered_resolution.width as f32,
                            scene.rendered_resolution.height as f32,
                        ),
                        fit_render_to_window(
                            screen_width,
                            screen_height,
                            scene.rendered_resolution,
                        ),
                        Vector2::zero(),
                        0.0,
                        Color::WHITE,
                    );
                }
                let book_button = ui::book_button(screen_width, screen_height);
                ui::draw_book_button(&mut drawing, book_button);
                ui::draw_scene_audio_button(
                    &mut drawing,
                    ui::scene_audio_button(book_button),
                    ui::contains(ui::scene_audio_button(book_button), mouse),
                    audio_status,
                );
            }
        }
        if let Some(active_transition) = transition {
            match active_transition.kind {
                UiTransitionKind::PageTurnForward => {
                    ui::draw_page_turn(&mut drawing, true, active_transition.progress())
                }
                UiTransitionKind::PageTurnBackward => {
                    ui::draw_page_turn(&mut drawing, false, active_transition.progress())
                }
                UiTransitionKind::OpeningBook => {
                    ui::draw_book_transition(&mut drawing, active_transition.progress())
                }
                UiTransitionKind::ClosingBook => {
                    ui::draw_book_transition(&mut drawing, 1.0 - active_transition.progress())
                }
                UiTransitionKind::FadeToScene | UiTransitionKind::FadeToGallery => {
                    ui::draw_fade(&mut drawing, active_transition.progress())
                }
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
    ground: &GroundPlane,
    resolution: RenderResolution,
) -> Result<(scene::Scene, Bvh, Framebuffer), String> {
    if matches!(mode, RunMode::Compare) {
        let full = load_configured_scene(config, materials, false)?;
        let optimized = load_configured_scene(config, materials, true)?;
        print_scene_metrics(&optimized);
        let full_bvh = build_bvh(&full.primitives);
        let optimized_bvh = build_bvh(&optimized.primitives);
        print_bvh_metrics(&optimized_bvh);
        let (full_frame, full_time, _) = render_scene_at(
            camera,
            &full,
            &full_bvh,
            textures,
            lighting,
            config.sky,
            ground,
            0.0,
            RenderStrategy::BvhMultiThread,
            "full scene",
            resolution,
        );
        let (frame, optimized_time, _) = render_scene_at(
            camera,
            &optimized,
            &optimized_bvh,
            textures,
            lighting,
            config.sky,
            ground,
            0.0,
            RenderStrategy::BvhMultiThread,
            "optimized scene",
            resolution,
        );
        print_framebuffer_comparison("full vs optimized", &full_frame, &frame)?;
        println!("Render times: full {full_time:.2?}, optimized {optimized_time:.2?}");
        return Ok((optimized, optimized_bvh, frame));
    }

    let scene = load_configured_scene(config, materials, !matches!(mode, RunMode::Full))?;
    print_scene_metrics(&scene);
    let bvh = build_bvh(&scene.primitives);
    print_bvh_metrics(&bvh);
    if matches!(mode, RunMode::BenchmarkQuality) {
        let interactive_resolution = RenderResolution {
            width: RENDER_QUALITY.interactive_max_width,
            height: RENDER_QUALITY.interactive_max_height,
        };
        let final_resolution = RenderResolution {
            width: RENDER_QUALITY.final_max_width,
            height: RENDER_QUALITY.final_max_height,
        };
        let (_, interactive_time, workers) = render_scene_at(
            camera,
            &scene,
            &bvh,
            textures,
            lighting,
            config.sky,
            ground,
            0.0,
            RenderStrategy::BvhMultiThread,
            "quality benchmark interactive",
            interactive_resolution,
        );
        let (frame, final_time, _) = render_scene_at(
            camera,
            &scene,
            &bvh,
            textures,
            lighting,
            config.sky,
            ground,
            0.0,
            RenderStrategy::BvhMultiThread,
            "quality benchmark final",
            final_resolution,
        );
        println!(
            "Quality benchmark: interactive {interactive_time:.2?}, final {final_time:.2?}, {workers} workers"
        );
        return Ok((scene, bvh, frame));
    }
    if matches!(mode, RunMode::BenchmarkAccelerators) {
        let (linear, linear_time, _) = render_scene_at(
            camera,
            &scene,
            &bvh,
            textures,
            lighting,
            config.sky,
            ground,
            0.0,
            RenderStrategy::LinearSingleThread,
            "linear single-thread",
            resolution,
        );
        let (single, single_time, _) = render_scene_at(
            camera,
            &scene,
            &bvh,
            textures,
            lighting,
            config.sky,
            ground,
            0.0,
            RenderStrategy::BvhSingleThread,
            "BVH single-thread",
            resolution,
        );
        print_framebuffer_comparison("linear vs BVH", &linear, &single)?;
        let (frame, multi_time, workers) = render_scene_at(
            camera,
            &scene,
            &bvh,
            textures,
            lighting,
            config.sky,
            ground,
            0.0,
            RenderStrategy::BvhMultiThread,
            "BVH multi-thread",
            resolution,
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
    let frame = render_scene_at(
        camera,
        &scene,
        &bvh,
        textures,
        lighting,
        config.sky,
        ground,
        0.0,
        RenderStrategy::BvhMultiThread,
        label,
        resolution,
    )
    .0;
    Ok((scene, bvh, frame))
}

fn load_configured_scene(
    config: &SceneConfig,
    materials: &MaterialCatalog,
    cull_fully_enclosed: bool,
) -> Result<scene::Scene, String> {
    let mut scene = scene::load(
        config.scene_path,
        |name| materials.for_block(name),
        cull_fully_enclosed,
    )?;
    if config.rotate_y_180 {
        scene.rotate_y_180();
    }
    Ok(scene)
}

fn print_camera_state(camera: &Camera) {
    println!(
        "Camera: yaw {:.2} deg, pitch {:.2} deg, distance {:.2}",
        camera.yaw_degrees(),
        camera.pitch_degrees(),
        camera.radius()
    );
}

fn print_teaser_camera_snapshot(scene_id: &str, camera: &Camera) {
    let position = camera.position();
    let target = camera.target();
    println!(
        "\n=== TEASER CAMERA SNAPSHOT ===\nscene: {scene_id}\nposition: Vec3::new({:?}, {:?}, {:?})\ntarget: Vec3::new({:?}, {:?}, {:?})\nradius: {:?}\nyaw: {:?}\npitch: {:?}\nfov: {:?}\n\nteaser(\n    \"{scene_id}\",\n    \"{}\",\n    Vec3 {{ x: {:?}, y: {:?}, z: {:?} }},\n    {:?},\n    {:?},\n    {:?},\n    0.06,\n),\n==============================\n",
        position.x,
        position.y,
        position.z,
        target.x,
        target.y,
        target.z,
        camera.radius(),
        camera.yaw(),
        camera.pitch(),
        camera.fov_degrees(),
        teaser_folder(scene_id),
        target.x,
        target.y,
        target.z,
        camera.radius(),
        camera.yaw(),
        camera.pitch(),
    );
}

fn teaser_folder(scene_id: &str) -> &str {
    match scene_id {
        "black_clover_skull" => "black_clover",
        other => other,
    }
}

fn key_axis(raylib: &RaylibHandle, negative: KeyboardKey, positive: KeyboardKey) -> f32 {
    raylib.is_key_down(positive) as u8 as f32 - raylib.is_key_down(negative) as u8 as f32
}

fn target_resolution(
    window_width: i32,
    window_height: i32,
    quality: QualityLevel,
) -> RenderResolution {
    let (max_width, max_height) = match quality {
        QualityLevel::Interactive => (
            RENDER_QUALITY.interactive_max_width,
            RENDER_QUALITY.interactive_max_height,
        ),
        QualityLevel::Final => (
            RENDER_QUALITY.final_max_width,
            RENDER_QUALITY.final_max_height,
        ),
    };
    let available_width = window_width.max(RENDER_ASPECT_WIDTH as i32) as u32;
    let available_height = window_height.max(RENDER_ASPECT_HEIGHT as i32) as u32;
    let scale = (available_width / RENDER_ASPECT_WIDTH)
        .min(available_height / RENDER_ASPECT_HEIGHT)
        .min(max_width / RENDER_ASPECT_WIDTH)
        .min(max_height / RENDER_ASPECT_HEIGHT)
        .max(1);
    RenderResolution {
        width: RENDER_ASPECT_WIDTH * scale,
        height: RENDER_ASPECT_HEIGHT * scale,
    }
}

fn settled_quality(idle_time: Duration) -> Option<QualityLevel> {
    (idle_time >= RENDER_QUALITY.settle_delay).then_some(QualityLevel::Final)
}

fn fit_render_to_window(
    window_width: i32,
    window_height: i32,
    resolution: RenderResolution,
) -> Rectangle {
    let scale = (window_width as f32 / resolution.width as f32)
        .min(window_height as f32 / resolution.height as f32);
    let width = resolution.width as f32 * scale;
    let height = resolution.height as f32 * scale;
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
    let mut generate_menu_backgrounds = None;
    let mut teaser_camera_scene_id = None;
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--scene" => {
                initial_scene_id = Some(arguments.next().ok_or("--scene requires a scene id")?)
            }
            "--no-culling" => mode = RunMode::Full,
            "--compare-culling" => mode = RunMode::Compare,
            "--benchmark-accelerators" => mode = RunMode::BenchmarkAccelerators,
            "--benchmark-quality" => mode = RunMode::BenchmarkQuality,
            "--generate-menu-backgrounds" => generate_menu_backgrounds = Some(false),
            "--generate-menu-background-samples" => generate_menu_backgrounds = Some(true),
            "--teaser-camera" => {
                let scene_id = arguments
                    .next()
                    .ok_or("--teaser-camera requires a scene id")?;
                initial_scene_id = Some(scene_id.clone());
                teaser_camera_scene_id = Some(scene_id);
            }
            _ => return Err(format!("unknown argument '{argument}'")),
        }
    }
    if initial_scene_id.is_none() && !matches!(mode, RunMode::Optimized) {
        initial_scene_id = Some("black_clover_skull".to_owned());
    }
    Ok(AppOptions {
        mode,
        initial_scene_id,
        generate_menu_backgrounds,
        teaser_camera_scene_id,
    })
}

fn generate_menu_backgrounds(samples_only: bool) -> Result<(), String> {
    let resolution = RenderResolution {
        width: menu_background::WIDTH,
        height: menu_background::HEIGHT,
    };
    let frame_count = if samples_only {
        3
    } else {
        menu_background::FRAMES_PER_SCENE
    };
    let started = Instant::now();
    for teaser in menu_background::TEASERS {
        let config = find_scene(teaser.scene_id)?;
        if samples_only {
            menu_background::prepare_sample_output(teaser)?;
        } else {
            menu_background::prepare_output(teaser)?;
        }
        let (materials, textures) = MaterialCatalog::load(&config.textures, &config.materials)?;
        let scene = load_configured_scene(&config, &materials, true)?;
        let bvh = build_bvh(&scene.primitives);
        let ground = GroundPlane::from_config(config.ground)?;
        println!(
            "Generating {} teaser frames for {}...",
            frame_count, config.id
        );
        for output_frame in 0..frame_count {
            let sequence_frame = if samples_only {
                [
                    0,
                    menu_background::FRAMES_PER_SCENE / 2,
                    menu_background::FRAMES_PER_SCENE - 1,
                ][output_frame]
            } else {
                output_frame
            };
            let camera_config = if samples_only {
                teaser.camera_at_phase([0.0, 0.5, 1.0][output_frame])
            } else {
                teaser.camera_for_frame(sequence_frame, menu_background::FRAMES_PER_SCENE)
            };
            let camera = Camera::new(camera_config);
            let render_time = sequence_frame as f32 / menu_background::FPS as f32;
            let framebuffer = render_scene_at(
                &camera,
                &scene,
                &bvh,
                &textures,
                &config.lighting,
                config.sky,
                &ground,
                render_time,
                RenderStrategy::BvhMultiThread,
                "menu teaser",
                resolution,
            )
            .0;
            let path = if samples_only {
                teaser.sample_path(["start", "middle", "end"][output_frame])
            } else {
                teaser.frame_path(output_frame)
            };
            framebuffer.to_image().export_image(&path);
        }
    }
    println!("Menu backgrounds generated in {:.2?}", started.elapsed());
    Ok(())
}

fn build_bvh(primitives: &[crate::cube::Cube]) -> Bvh {
    let started = Instant::now();
    let bvh = Bvh::build(primitives);
    println!("BVH built in {:.2?}", started.elapsed());
    bvh
}

fn render_scene_at(
    camera: &Camera,
    scene: &scene::Scene,
    bvh: &Bvh,
    textures: &[Texture],
    lighting: &LightingConfig,
    sky: SkyType,
    ground: &GroundPlane,
    render_time_seconds: f32,
    strategy: RenderStrategy,
    label: &str,
    resolution: RenderResolution,
) -> (Framebuffer, std::time::Duration, usize) {
    let mut framebuffer = Framebuffer::new(resolution.width, resolution.height, Color::BLACK);
    println!(
        "Rendering {label} at {} x {} ({} primary pixels)...",
        resolution.width,
        resolution.height,
        resolution.pixel_count()
    );
    let started = Instant::now();
    let workers = render::render(
        &mut framebuffer,
        camera,
        &scene.primitives,
        bvh,
        textures,
        lighting,
        sky,
        ground,
        render_time_seconds,
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
        let rectangle = fit_render_to_window(
            1600,
            800,
            RenderResolution {
                width: 1280,
                height: 720,
            },
        );
        assert!(((rectangle.width / rectangle.height) - (16.0 / 9.0)).abs() < 0.00001);
        assert!(rectangle.x > 0.0);
        assert!(rectangle.y.abs() < 0.0001);
    }

    #[test]
    fn render_rectangle_preserves_aspect_ratio_in_tall_window() {
        let rectangle = fit_render_to_window(
            800,
            1000,
            RenderResolution {
                width: 1280,
                height: 720,
            },
        );
        assert!(((rectangle.width / rectangle.height) - (16.0 / 9.0)).abs() < 0.00001);
        assert_eq!(rectangle.x, 0.0);
        assert!(rectangle.y > 0.0);
    }

    #[test]
    fn quality_resolutions_preserve_aspect_ratio_and_caps() {
        assert_eq!(
            target_resolution(1920, 1080, QualityLevel::Interactive),
            RenderResolution {
                width: 640,
                height: 360
            }
        );
        assert_eq!(
            target_resolution(1920, 1080, QualityLevel::Final),
            RenderResolution {
                width: 1280,
                height: 720
            }
        );
        assert_eq!(
            target_resolution(800, 1000, QualityLevel::Final),
            RenderResolution {
                width: 800,
                height: 450
            }
        );
    }

    #[test]
    fn final_quality_waits_for_debounce() {
        assert_eq!(settled_quality(Duration::from_millis(199)), None);
        assert_eq!(
            settled_quality(Duration::from_millis(200)),
            Some(QualityLevel::Final)
        );
    }
}
