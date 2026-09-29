use std::path::Path;

use raylib::prelude::{Color, Image};

#[derive(Debug)]
pub struct Texture {
    width: u32,
    frame_height: u32,
    frame_count: usize,
    animation_fps: f32,
    pixels: Vec<Color>,
}

impl Texture {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        Self::load_rows(path, None, 0.0)
    }

    pub fn load_animated(
        path: impl AsRef<Path>,
        frame_height: u32,
        fps: f32,
    ) -> Result<Self, String> {
        Self::load_rows(path, Some(frame_height), fps)
    }

    fn load_rows(
        path: impl AsRef<Path>,
        frame_height: Option<u32>,
        animation_fps: f32,
    ) -> Result<Self, String> {
        let path = path.as_ref();
        let path_text = path.to_string_lossy();
        let image = Image::load_image(&path_text)
            .map_err(|error| format!("could not load texture '{}': {error}", path.display()))?;
        let width = image.width() as u32;
        let image_height = image.height() as u32;
        let frame_height = frame_height.unwrap_or(image_height);

        if frame_height == 0 || frame_height > image_height || image_height % frame_height != 0 {
            return Err(format!(
                "texture '{}' has height {image_height}, incompatible with frame height {frame_height}",
                path.display()
            ));
        }
        if animation_fps < 0.0 || !animation_fps.is_finite() {
            return Err("texture animation FPS must be finite and non-negative".to_string());
        }

        let pixels = image.get_image_data().to_vec();

        Ok(Self {
            width,
            frame_height,
            frame_count: (image_height / frame_height) as usize,
            animation_fps,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn sample_frame(&self, u: f32, v: f32, frame: usize) -> Color {
        let wrapped_u = u.rem_euclid(1.0);
        let wrapped_v = v.rem_euclid(1.0);
        let x = (wrapped_u * self.width() as f32) as u32;
        let frame = frame % self.frame_count;
        let y = frame as u32 * self.frame_height + (wrapped_v * self.frame_height as f32) as u32;

        self.pixels[(y * self.width() + x) as usize]
    }

    pub fn is_animated(&self) -> bool {
        self.frame_count > 1 && self.animation_fps > 0.0
    }

    pub fn frame_index(&self, elapsed_seconds: f32) -> usize {
        if !self.is_animated() {
            return 0;
        }
        ((elapsed_seconds.max(0.0) * self.animation_fps).floor() as usize) % self.frame_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_neighbor_sampling_repeats_uv_coordinates() {
        let texture = Texture {
            width: 2,
            frame_height: 2,
            frame_count: 1,
            animation_fps: 0.0,
            pixels: vec![Color::RED, Color::GREEN, Color::BLUE, Color::WHITE],
        };

        assert_eq!(texture.sample_frame(0.1, 0.1, 0), Color::RED);
        assert_eq!(texture.sample_frame(0.9, 0.1, 0), Color::GREEN);
        assert_eq!(texture.sample_frame(1.1, -0.9, 0), Color::RED);
    }

    #[test]
    fn nearest_neighbor_handles_texel_and_wrap_boundaries() {
        let texture = Texture {
            width: 2,
            frame_height: 2,
            frame_count: 1,
            animation_fps: 0.0,
            pixels: vec![Color::RED, Color::GREEN, Color::BLUE, Color::WHITE],
        };

        assert_eq!(texture.sample_frame(0.4999, 0.4999, 0), Color::RED);
        assert_eq!(texture.sample_frame(0.5, 0.5, 0), Color::WHITE);
        assert_eq!(texture.sample_frame(0.9999, 0.9999, 0), Color::WHITE);
        assert_eq!(texture.sample_frame(1.0, 1.0, 0), Color::RED);
    }

    #[test]
    fn animated_frame_selection_is_deterministic_and_wraps() {
        let texture = Texture {
            width: 1,
            frame_height: 1,
            frame_count: 3,
            animation_fps: 10.0,
            pixels: vec![Color::RED, Color::GREEN, Color::BLUE],
        };
        assert_eq!(texture.frame_index(0.0), 0);
        assert_eq!(texture.frame_index(0.1), 1);
        assert_eq!(texture.frame_index(0.3), 0);
        assert_eq!(texture.frame_index(1.27), texture.frame_index(1.27));
        assert_eq!(texture.sample_frame(0.0, 0.0, 2), Color::BLUE);
    }
}
