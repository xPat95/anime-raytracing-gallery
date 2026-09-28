use raylib::prelude::Color;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub albedo: Color,
}

impl Material {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}
