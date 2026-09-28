use crate::{material::Material, vec3::Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Intersect {
    pub distance: f32,
    pub is_intersecting: bool,
    pub normal: Vec3,
    pub u: f32,
    pub v: f32,
    pub material: Material,
}

impl Intersect {
    pub fn new(distance: f32, normal: Vec3, u: f32, v: f32, material: Material) -> Self {
        Self {
            distance,
            is_intersecting: true,
            normal,
            u,
            v,
            material,
        }
    }

    pub fn empty() -> Self {
        Self {
            distance: f32::INFINITY,
            is_intersecting: false,
            normal: Vec3::default(),
            u: 0.0,
            v: 0.0,
            material: Material::new(raylib::prelude::Color::BLANK),
        }
    }
}
