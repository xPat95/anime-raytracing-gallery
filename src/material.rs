use raylib::prelude::Color;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub albedo: Color,
    pub texture_index: Option<usize>,
}

impl Material {
    pub fn new(albedo: Color) -> Self {
        Self {
            albedo,
            texture_index: None,
        }
    }

    pub fn textured(texture_index: usize) -> Self {
        Self {
            albedo: Color::WHITE,
            texture_index: Some(texture_index),
        }
    }
}
