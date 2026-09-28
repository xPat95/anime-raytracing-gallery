use raylib::prelude::Color;

use crate::vec3::Vec3;

#[derive(Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

#[derive(Clone, Copy)]
pub struct LightingConfig {
    pub light: PointLight,
    pub ambient_intensity: f32,
    pub shadow_bias: f32,
    pub phong_shininess: f32,
}
