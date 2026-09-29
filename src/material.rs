use raylib::prelude::Color;

const TRANSPARENCY_EPSILON: f32 = 1.0e-6;

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub base_color: Color,
    pub texture_index: Option<usize>,
    pub albedo: f32,
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub ior: Option<f32>,
    pub emission_tint: Color,
    pub emission_strength: f32,
}

impl Material {
    pub fn new(base_color: Color) -> Self {
        Self {
            base_color,
            texture_index: None,
            albedo: 1.0,
            specular: 0.0,
            transparency: 0.0,
            reflectivity: 0.0,
            ior: None,
            emission_tint: Color::WHITE,
            emission_strength: 0.0,
        }
    }

    pub fn textured(
        texture_index: usize,
        albedo: f32,
        specular: f32,
        transparency: f32,
        reflectivity: f32,
        ior: Option<f32>,
    ) -> Self {
        let material = Self {
            base_color: Color::WHITE,
            texture_index: Some(texture_index),
            albedo,
            specular,
            transparency,
            reflectivity,
            ior,
            emission_tint: Color::WHITE,
            emission_strength: 0.0,
        };
        material.validate();
        material
    }

    pub fn is_opaque(&self) -> bool {
        self.transparency <= TRANSPARENCY_EPSILON
    }

    fn validate(&self) {
        for (name, value) in [
            ("albedo", self.albedo),
            ("specular", self.specular),
            ("transparency", self.transparency),
            ("reflectivity", self.reflectivity),
        ] {
            assert!(
                (0.0..=1.0).contains(&value),
                "material {name} must be between 0.0 and 1.0"
            );
        }
        if let Some(ior) = self.ior {
            assert!(
                ior.is_finite() && ior > 0.0,
                "material IOR must be positive"
            );
        }
        assert!(
            self.emission_strength.is_finite() && self.emission_strength >= 0.0,
            "material emission strength must be finite and non-negative"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_uses_transparency_with_a_small_tolerance() {
        let mut material = Material::new(Color::WHITE);
        assert!(material.is_opaque());

        material.transparency = TRANSPARENCY_EPSILON;
        assert!(material.is_opaque());

        material.transparency = 0.01;
        assert!(!material.is_opaque());
    }

    #[test]
    fn emission_does_not_change_optical_properties() {
        let mut material = Material::new(Color::WHITE);
        material.transparency = 0.35;
        material.reflectivity = 0.12;
        material.ior = Some(1.5);
        material.emission_tint = Color::GOLD;
        material.emission_strength = 0.5;

        assert!(!material.is_opaque());
        assert_eq!(material.transparency, 0.35);
        assert_eq!(material.reflectivity, 0.12);
        assert_eq!(material.ior, Some(1.5));
    }

    #[test]
    fn opaque_emissive_material_remains_opaque() {
        let mut material = Material::new(Color::WHITE);
        material.emission_strength = 0.8;
        assert!(material.is_opaque());
    }
}
