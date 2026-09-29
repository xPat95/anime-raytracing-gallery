use raylib::prelude::Color;

use crate::{intersect::Intersect, material::Material, vec3::Vec3};

const PLANE_EPSILON: f32 = 0.0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundKind {
    Solid,
    Water,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolidGroundStyle {
    DarkRock,
    Grass,
    Stone,
    VividGrass,
}

#[derive(Clone, Copy)]
pub struct GroundConfig {
    pub kind: GroundKind,
    pub solid_style: Option<SolidGroundStyle>,
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
    solid_style: Option<SolidGroundStyle>,
    height: f32,
    material: Material,
}

impl GroundPlane {
    pub fn new(
        kind: GroundKind,
        solid_style: Option<SolidGroundStyle>,
        height: f32,
        material: Material,
    ) -> Self {
        Self {
            kind,
            solid_style,
            height,
            material,
        }
    }

    pub fn from_config(config: GroundConfig) -> Result<Self, String> {
        match config.kind {
            GroundKind::Solid if config.solid_style.is_none() => {
                return Err("solid ground requires a procedural style".to_string());
            }
            GroundKind::Solid if config.transparency > 0.0 || config.ior.is_some() => {
                return Err("solid ground cannot be transparent or refractive".to_string());
            }
            GroundKind::Water if config.transparency <= 0.0 || config.ior.is_none() => {
                return Err("water ground must be transparent and refractive".to_string());
            }
            GroundKind::Water if config.solid_style.is_some() => {
                return Err("water ground cannot use a solid procedural style".to_string());
            }
            _ => {}
        }
        let mut material = Material::new(config.tint);
        material.albedo = config.albedo;
        material.specular = config.specular;
        material.transparency = config.transparency;
        material.reflectivity = config.reflectivity;
        material.ior = config.ior;
        Ok(Self::new(
            config.kind,
            config.solid_style,
            config.height,
            material,
        ))
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

    pub fn surface_color(&self, point: Vec3, base_color: Vec3) -> Vec3 {
        let Some(style) = self.solid_style else {
            return base_color;
        };
        let (scale, detail_scale, seed, dark, light) = match style {
            SolidGroundStyle::DarkRock => (
                0.022,
                0.085,
                11.0,
                Vec3::new(0.56, 0.59, 0.61),
                Vec3::new(1.24, 1.20, 1.15),
            ),
            SolidGroundStyle::Stone => (
                0.019,
                0.072,
                37.0,
                Vec3::new(0.70, 0.71, 0.73),
                Vec3::new(1.22, 1.20, 1.17),
            ),
            SolidGroundStyle::Grass => (
                0.016,
                0.065,
                73.0,
                Vec3::new(0.66, 0.78, 0.61),
                Vec3::new(1.16, 1.23, 0.96),
            ),
            SolidGroundStyle::VividGrass => (
                0.018,
                0.078,
                109.0,
                Vec3::new(0.70, 0.82, 0.60),
                Vec3::new(1.18, 1.27, 0.91),
            ),
        };
        let broad = value_noise(point.x * scale, point.z * scale, seed);
        let detail = value_noise(
            point.x * detail_scale + 19.7,
            point.z * detail_scale - 8.3,
            seed + 41.0,
        );
        let variation = (broad * 0.76 + detail * 0.24).clamp(0.0, 1.0);
        component_multiply(base_color, mix(dark, light, variation))
    }
}

fn value_noise(x: f32, z: f32, seed: f32) -> f32 {
    let cell_x = x.floor();
    let cell_z = z.floor();
    let local_x = smooth_curve(x - cell_x);
    let local_z = smooth_curve(z - cell_z);
    let bottom = lerp(
        hash(cell_x, cell_z, seed),
        hash(cell_x + 1.0, cell_z, seed),
        local_x,
    );
    let top = lerp(
        hash(cell_x, cell_z + 1.0, seed),
        hash(cell_x + 1.0, cell_z + 1.0, seed),
        local_x,
    );
    lerp(bottom, top, local_z)
}

fn hash(x: f32, z: f32, seed: f32) -> f32 {
    ((x * 127.1 + z * 311.7 + seed * 74.7).sin() * 43_758.547)
        .fract()
        .abs()
}

fn smooth_curve(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(left: f32, right: f32, amount: f32) -> f32 {
    left * (1.0 - amount) + right * amount
}

fn mix(left: Vec3, right: Vec3, amount: f32) -> Vec3 {
    left * (1.0 - amount) + right * amount
}

fn component_multiply(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x * right.x, left.y * right.y, left.z * right.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane(kind: GroundKind) -> GroundPlane {
        GroundPlane::new(
            kind,
            (kind == GroundKind::Solid).then_some(SolidGroundStyle::DarkRock),
            -2.0,
            Material::new(Color::WHITE),
        )
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
    fn solid_surface_is_deterministic_and_style_specific() {
        let plane = plane(GroundKind::Solid);
        let point = Vec3::new(12.0, -2.0, -31.0);
        let color = plane.surface_color(point, Vec3::new(0.4, 0.4, 0.4));
        assert_eq!(color, plane.surface_color(point, Vec3::new(0.4, 0.4, 0.4)));
        let stone = GroundPlane::new(
            GroundKind::Solid,
            Some(SolidGroundStyle::Stone),
            -2.0,
            Material::new(Color::WHITE),
        );
        assert_ne!(color, stone.surface_color(point, Vec3::new(0.4, 0.4, 0.4)));
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

    #[test]
    fn water_bypasses_solid_surface_variation() {
        let water = plane(GroundKind::Water);
        let base = Vec3::new(0.2, 0.4, 0.8);
        assert_eq!(water.surface_color(Vec3::new(18.0, -2.0, 31.0), base), base);
    }
}
