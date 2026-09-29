use raylib::prelude::Color;

use crate::{intersect::Intersect, material::Material, vec3::Vec3};

const PLANE_EPSILON: f32 = 0.0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundKind {
    Solid,
    Water,
}

#[derive(Clone, Copy)]
pub struct GroundConfig {
    pub kind: GroundKind,
    pub height: f32,
    pub tint: Color,
    pub albedo: f32,
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub ior: Option<f32>,
}

#[derive(Clone, Copy)]
pub struct GroundPlane {
    kind: GroundKind,
    height: f32,
    material: Material,
}

impl GroundPlane {
    pub fn new(kind: GroundKind, height: f32, material: Material) -> Self {
        Self {
            kind,
            height,
            material,
        }
    }

    pub fn from_config(config: GroundConfig) -> Result<Self, String> {
        match config.kind {
            GroundKind::Solid if config.transparency > 0.0 || config.ior.is_some() => {
                return Err("solid ground cannot be transparent or refractive".to_string());
            }
            GroundKind::Water if config.transparency <= 0.0 || config.ior.is_none() => {
                return Err("water ground must be transparent and refractive".to_string());
            }
            _ => {}
        }
        let mut material = Material::new(config.tint);
        material.albedo = config.albedo;
        material.specular = config.specular;
        material.transparency = config.transparency;
        material.reflectivity = config.reflectivity;
        material.ior = config.ior;
        Ok(Self::new(config.kind, config.height, material))
    }

    pub fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        if ray_direction.y.abs() < PLANE_EPSILON {
            return Intersect::empty();
        }
        let distance = (self.height - ray_origin.y) / ray_direction.y;
        if distance <= PLANE_EPSILON {
            return Intersect::empty();
        }
        Intersect::new(distance, self.geometric_normal(), 0.0, 0.0, self.material)
    }

    pub fn geometric_normal(&self) -> Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }

    pub fn shading_normal(&self, point: Vec3) -> Vec3 {
        if self.kind == GroundKind::Solid {
            return self.geometric_normal();
        }
        let slope_x = 0.055 * (point.x * 0.075 + point.z * 0.031).cos()
            + 0.035 * (point.x * 0.043 - point.z * 0.067 + 1.7).cos()
            + 0.025 * (point.x * 0.112 + point.z * 0.019 + 0.4).sin()
            + 0.018 * (point.x * 0.026 - point.z * 0.094 + 2.3).sin();
        let slope_z = 0.055 * (point.x * 0.031 + point.z * 0.075).cos()
            - 0.035 * (point.x * 0.067 - point.z * 0.043 + 1.7).cos()
            + 0.025 * (point.x * 0.019 + point.z * 0.112 + 0.4).sin()
            - 0.018 * (point.x * 0.094 - point.z * 0.026 + 2.3).sin();
        Vec3::new(slope_x, 1.0, slope_z).normalize()
    }

    pub fn color_variation(&self, point: Vec3) -> f32 {
        if self.kind == GroundKind::Water {
            return 1.0;
        }
        let broad = (point.x * 0.035 + point.z * 0.021).sin();
        let crossing = (point.x * 0.017 - point.z * 0.029 + 1.3).cos();
        (1.0 + broad * 0.025 + crossing * 0.018).clamp(0.94, 1.06)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane(kind: GroundKind) -> GroundPlane {
        GroundPlane::new(kind, -2.0, Material::new(Color::WHITE))
    }

    #[test]
    fn downward_ray_hits_plane_at_expected_distance() {
        let hit = plane(GroundKind::Solid)
            .ray_intersect(&Vec3::new(3.25, 2.0, -4.75), &Vec3::new(0.0, -1.0, 0.0));
        assert!(hit.is_intersecting);
        assert_eq!(hit.distance, 4.0);
        assert_eq!(hit.normal, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn parallel_and_away_rays_do_not_hit_plane() {
        let plane = plane(GroundKind::Solid);
        assert!(
            !plane
                .ray_intersect(&Vec3::default(), &Vec3::new(1.0, 0.0, 0.0))
                .is_intersecting
        );
        assert!(
            !plane
                .ray_intersect(&Vec3::default(), &Vec3::new(0.0, 1.0, 0.0))
                .is_intersecting
        );
    }

    #[test]
    fn solid_variation_is_subtle_and_deterministic() {
        let plane = plane(GroundKind::Solid);
        let point = Vec3::new(12.0, -2.0, -31.0);
        let variation = plane.color_variation(point);
        assert_eq!(variation, plane.color_variation(point));
        assert!((0.94..=1.06).contains(&variation));
    }

    #[test]
    fn water_normals_vary_remain_normalized_and_point_up() {
        let plane = plane(GroundKind::Water);
        let first = plane.shading_normal(Vec3::new(0.0, -2.0, 0.0));
        let second = plane.shading_normal(Vec3::new(23.0, -2.0, -17.0));
        assert_ne!(first, second);
        assert!((first.length() - 1.0).abs() < 0.0001);
        assert!((second.length() - 1.0).abs() < 0.0001);
        assert!(first.y > 0.98 && second.y > 0.98);
        assert_eq!(first, plane.shading_normal(Vec3::new(0.0, -2.0, 0.0)));
    }
}
