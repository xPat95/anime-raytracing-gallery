use raylib::prelude::Color;

use crate::vec3::Vec3;

pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

pub struct LightingConfig {
    pub light: PointLight,
    pub ambient_intensity: f32,
    pub shadow_bias: f32,
    pub phong_shininess: f32,
}

impl LightingConfig {
    pub fn scene() -> Self {
        Self {
            light: PointLight {
                position: Vec3::new(-90.0, 125.0, 140.0),
                color: Color::new(255, 244, 224, 255),
                intensity: 1.15,
            },
            ambient_intensity: 0.10,
            shadow_bias: 0.001,
            phong_shininess: 32.0,
        }
    }
}
