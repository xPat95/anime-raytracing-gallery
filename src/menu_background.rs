use std::{fs, path::Path, time::SystemTime};

use raylib::prelude::*;

use crate::{camera::CameraConfig, vec3::Vec3};

pub const WIDTH: u32 = 800;
pub const HEIGHT: u32 = 450;
pub const FPS: usize = 10;
pub const FRAMES_PER_SCENE: usize = 50;
pub const SCENE_SECONDS: f32 = FRAMES_PER_SCENE as f32 / FPS as f32;
pub const CROSSFADE_SECONDS: f32 = 0.85;

pub const SPLASHES: [&str; 8] = [
    "Now with Ray Tracing!",
    "Powered by Rust!",
    "Recursive rays!",
    "5 worlds!",
    "100% cubes!",
    "BVH accelerated!",
    "Anime + Minecraft!",
    "Lapras approved!",
];

#[derive(Clone, Copy)]
pub struct TeaserSpec {
    pub scene_id: &'static str,
    pub folder: &'static str,
    pub camera: CameraConfig,
    pub orbit_radians: f32,
}

pub const TEASERS: [TeaserSpec; 5] = [
    teaser(
        "black_clover_skull",
        "black_clover",
        Vec3 {
            x: -1.9771733,
            y: -2.133336,
            z: 4.199544,
        },
        44.499977,
        0.32497835,
        0.4907596,
        0.06,
    ),
    teaser(
        "shenlong",
        "shenlong",
        Vec3 {
            x: -15.668148,
            y: -61.416183,
            z: -7.981523,
        },
        26.89743,
        6.017248,
        -0.058275938,
        0.06,
    ),
    teaser(
        "kurama",
        "kurama",
        Vec3 {
            x: -27.088326,
            y: -28.761879,
            z: 12.898168,
        },
        98.70358,
        0.56844753,
        -0.12836851,
        0.06,
    ),
    teaser(
        "pochita",
        "pochita",
        Vec3 {
            x: -22.825104,
            y: 23.965546,
            z: 15.7612,
        },
        49.65027,
        0.61253756,
        0.00949266,
        0.06,
    ),
    teaser(
        "lapras",
        "lapras",
        Vec3 {
            x: 0.16320339,
            y: -26.75723,
            z: 0.43818521,
        },
        50.713577,
        0.3565358,
        0.017473549,
        0.06,
    ),
];

const fn teaser(
    scene_id: &'static str,
    folder: &'static str,
    target: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    orbit_radians: f32,
) -> TeaserSpec {
    TeaserSpec {
        scene_id,
        folder,
        camera: CameraConfig {
            target,
            yaw,
            pitch,
            radius,
            min_radius: 1.0,
            max_radius: 1_000.0,
        },
        orbit_radians,
    }
}

impl TeaserSpec {
    pub fn camera_for_frame(self, frame: usize, frame_count: usize) -> CameraConfig {
        let phase = if frame_count <= 1 {
            0.5
        } else {
            frame as f32 / (frame_count - 1) as f32
        };
        let eased_phase = phase * phase * (3.0 - 2.0 * phase);
        self.camera_at_phase(eased_phase)
    }

    pub fn camera_at_phase(self, phase: f32) -> CameraConfig {
        CameraConfig {
            yaw: self.camera.yaw + phase.clamp(0.0, 1.0) * self.orbit_radians,
            ..self.camera
        }
    }

    pub fn frame_path(self, frame: usize) -> String {
        format!(
            "assets/backgrounds/menu/{}/frame_{frame:03}.jpg",
            self.folder
        )
    }

    pub fn sample_path(self, label: &str) -> String {
        format!(
            "assets/backgrounds/menu_samples/{}/{label}.jpg",
            self.folder
        )
    }
}

pub fn prepare_output(spec: TeaserSpec) -> Result<(), String> {
    let directory = format!("assets/backgrounds/menu/{}", spec.folder);
    fs::create_dir_all(&directory)
        .map_err(|error| format!("could not create '{directory}': {error}"))?;
    Ok(())
}

pub fn prepare_sample_output(spec: TeaserSpec) -> Result<(), String> {
    let directory = format!("assets/backgrounds/menu_samples/{}", spec.folder);
    fs::create_dir_all(&directory)
        .map_err(|error| format!("could not create '{directory}': {error}"))
}

pub fn choose_splash() -> &'static str {
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |time| time.as_nanos());
    SPLASHES[seed as usize % SPLASHES.len()]
}

pub struct MenuBackground {
    sequences: Vec<Vec<Texture2D>>,
    elapsed: f32,
}

impl MenuBackground {
    pub fn load(raylib: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let sequences = TEASERS
            .iter()
            .map(|spec| {
                (0..FRAMES_PER_SCENE)
                    .filter_map(|frame| {
                        let path = spec.frame_path(frame);
                        if !Path::new(&path).exists() {
                            return None;
                        }
                        match raylib.load_texture(thread, &path) {
                            Ok(texture) => {
                                texture.set_texture_filter(
                                    thread,
                                    TextureFilter::TEXTURE_FILTER_BILINEAR,
                                );
                                Some(texture)
                            }
                            Err(error) => {
                                eprintln!("Menu background skipped '{path}': {error}");
                                None
                            }
                        }
                    })
                    .collect()
            })
            .collect();
        Self {
            sequences,
            elapsed: 0.0,
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.elapsed += delta.max(0.0);
    }

    pub fn timeline(&self) -> (usize, usize, usize, f32) {
        timeline_state(self.elapsed, self.sequences.len())
    }

    pub fn loaded_frames(&self) -> usize {
        self.sequences.iter().map(Vec::len).sum()
    }

    pub fn approximate_gpu_bytes(&self) -> usize {
        self.loaded_frames() * WIDTH as usize * HEIGHT as usize * 4
    }

    pub fn draw(&self, drawing: &mut RaylibDrawHandle<'_>, overlay_alpha: u8) {
        drawing.clear_background(Color::new(12, 17, 24, 255));
        let (current, next, frame, blend) = self.timeline();
        self.draw_frame(drawing, current, frame, Color::WHITE);
        if blend > 0.0 {
            self.draw_frame(
                drawing,
                next,
                frame,
                Color::new(255, 255, 255, (blend * 255.0) as u8),
            );
        }
        drawing.draw_rectangle(
            0,
            0,
            drawing.get_screen_width(),
            drawing.get_screen_height(),
            Color::new(0, 0, 0, overlay_alpha),
        );
    }

    fn draw_frame(
        &self,
        drawing: &mut RaylibDrawHandle<'_>,
        sequence: usize,
        frame: usize,
        tint: Color,
    ) {
        let Some(texture) = self
            .sequences
            .get(sequence)
            .and_then(|frames| frames.get(frame % frames.len().max(1)))
        else {
            return;
        };
        let screen_width = drawing.get_screen_width() as f32;
        let screen_height = drawing.get_screen_height() as f32;
        let scale =
            (screen_width / texture.width as f32).max(screen_height / texture.height as f32);
        let source_width = screen_width / scale;
        let source_height = screen_height / scale;
        drawing.draw_texture_pro(
            texture,
            Rectangle::new(
                (texture.width as f32 - source_width) * 0.5,
                (texture.height as f32 - source_height) * 0.5,
                source_width,
                source_height,
            ),
            Rectangle::new(0.0, 0.0, screen_width, screen_height),
            Vector2::zero(),
            0.0,
            tint,
        );
    }
}

pub fn timeline_state(elapsed: f32, sequence_count: usize) -> (usize, usize, usize, f32) {
    let count = sequence_count.max(1);
    let cycle = elapsed.max(0.0) / SCENE_SECONDS;
    let current = cycle.floor() as usize % count;
    let local = (cycle.fract() * SCENE_SECONDS).min(SCENE_SECONDS);
    let frame = (local * FPS as f32).floor() as usize % FRAMES_PER_SCENE;
    let fade_start = SCENE_SECONDS - CROSSFADE_SECONDS;
    let blend = ((local - fade_start) / CROSSFADE_SECONDS).clamp(0.0, 1.0);
    (current, (current + 1) % count, frame, blend)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_advances_frames_and_wraps_scenes() {
        assert_eq!(timeline_state(0.0, 5).0, 0);
        assert_eq!(timeline_state(0.11, 5).2, 1);
        assert_eq!(timeline_state(SCENE_SECONDS, 5).0, 1);
        assert_eq!(timeline_state(SCENE_SECONDS * 5.0, 5).0, 0);
    }

    #[test]
    fn crossfade_runs_from_zero_to_one() {
        assert_eq!(timeline_state(0.0, 5).3, 0.0);
        assert!(timeline_state(SCENE_SECONDS - 0.1, 5).3 > 0.8);
    }

    #[test]
    fn teaser_cameras_do_not_change_scene_configs() {
        assert_eq!(TEASERS.len(), 5);
        assert_ne!(TEASERS[0].camera.radius, 249.89998);
    }
}
