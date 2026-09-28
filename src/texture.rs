use std::path::Path;

use raylib::prelude::{Color, Image};

#[derive(Debug)]
pub struct Texture {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
}

impl Texture {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        Self::load_rows(path, None)
    }

    pub fn load_first_frame(path: impl AsRef<Path>, frame_height: u32) -> Result<Self, String> {
        Self::load_rows(path, Some(frame_height))
    }

    fn load_rows(path: impl AsRef<Path>, frame_height: Option<u32>) -> Result<Self, String> {
        let path = path.as_ref();
        let path_text = path.to_string_lossy();
        let image = Image::load_image(&path_text)
            .map_err(|error| format!("could not load texture '{}': {error}", path.display()))?;
        let width = image.width() as u32;
        let image_height = image.height() as u32;
        let height = frame_height.unwrap_or(image_height);

        if height == 0 || height > image_height || image_height % height != 0 {
            return Err(format!(
                "texture '{}' has height {image_height}, incompatible with frame height {height}",
                path.display()
            ));
        }

        let pixels = image
            .get_image_data()
            .iter()
            .take((width * height) as usize)
            .copied()
            .collect();

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let wrapped_u = u.rem_euclid(1.0);
        let wrapped_v = v.rem_euclid(1.0);
        let x = (wrapped_u * self.width() as f32) as u32;
        let y = (wrapped_v * self.height() as f32) as u32;

        self.pixels[(y * self.width() + x) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_neighbor_sampling_repeats_uv_coordinates() {
        let texture = Texture {
            width: 2,
            height: 2,
            pixels: vec![Color::RED, Color::GREEN, Color::BLUE, Color::WHITE],
        };

        assert_eq!(texture.sample(0.1, 0.1), Color::RED);
        assert_eq!(texture.sample(0.9, 0.1), Color::GREEN);
        assert_eq!(texture.sample(1.1, -0.9), Color::RED);
    }
}
