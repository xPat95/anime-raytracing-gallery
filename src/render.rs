use raylib::prelude::Color;

use crate::{
    camera::Camera, cube::Cube, framebuffer::Framebuffer, intersect::Intersect,
    light::LightingConfig, ray_intersect::RayIntersect, texture::Texture, vec3::Vec3,
};

const BACKGROUND: Color = Color::new(18, 22, 30, 255);

pub fn render(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    cubes: &[Cube],
    textures: &[Texture],
    lighting: &LightingConfig,
) {
    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            let ray_origin = camera.position();
            let ray_direction =
                camera.ray_direction(x, y, framebuffer.width(), framebuffer.height());
            let closest_hit = cast_ray(&ray_origin, &ray_direction, cubes);
            let color = match closest_hit.is_intersecting {
                true => shade_hit(
                    closest_hit,
                    &ray_origin,
                    &ray_direction,
                    cubes,
                    textures,
                    lighting,
                ),
                false => BACKGROUND,
            };

            framebuffer.set_pixel(x, y, color);
        }
    }
}

fn cast_ray(ray_origin: &Vec3, ray_direction: &Vec3, cubes: &[Cube]) -> Intersect {
    let mut closest_hit = Intersect::empty();

    for cube in cubes {
        let hit = cube.ray_intersect(ray_origin, ray_direction);

        if hit.is_intersecting && hit.distance < closest_hit.distance {
            closest_hit = hit;
        }
    }

    closest_hit
}

fn shade_hit(
    hit: Intersect,
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    cubes: &[Cube],
    textures: &[Texture],
    lighting: &LightingConfig,
) -> Color {
    let surface_color = hit
        .material
        .texture_index
        .and_then(|index| textures.get(index))
        .map_or(hit.material.base_color, |texture| {
            texture.sample(hit.u, hit.v)
        });
    let hit_point = *ray_origin + *ray_direction * hit.distance;
    let normal = hit.normal.normalize();
    let to_light = lighting.light.position - hit_point;
    let light_distance = to_light.length();
    let light_direction = to_light / light_distance;
    let diffuse_angle = lambert(normal, light_direction);
    let shadow_origin = hit_point + normal * lighting.shadow_bias;
    let shadowed =
        diffuse_angle > 0.0 && is_shadowed(&shadow_origin, &light_direction, light_distance, cubes);

    let diffuse = if shadowed {
        0.0
    } else {
        diffuse_angle * hit.material.albedo * lighting.light.intensity
    };
    let specular = if shadowed || diffuse_angle == 0.0 {
        0.0
    } else {
        phong_specular(
            normal,
            light_direction,
            -*ray_direction,
            hit.material.specular,
            lighting.light.intensity,
            lighting.phong_shininess,
        )
    };
    let light_color = color_to_vec3(lighting.light.color);
    let base_color = color_to_vec3(surface_color);
    let lit_color = Vec3::new(
        base_color.x * (lighting.ambient_intensity + diffuse * light_color.x)
            + specular * light_color.x,
        base_color.y * (lighting.ambient_intensity + diffuse * light_color.y)
            + specular * light_color.y,
        base_color.z * (lighting.ambient_intensity + diffuse * light_color.z)
            + specular * light_color.z,
    );

    Color::new(
        (lit_color.x.clamp(0.0, 1.0) * 255.0) as u8,
        (lit_color.y.clamp(0.0, 1.0) * 255.0) as u8,
        (lit_color.z.clamp(0.0, 1.0) * 255.0) as u8,
        surface_color.a,
    )
}

fn lambert(normal: Vec3, light_direction: Vec3) -> f32 {
    normal.dot(light_direction).max(0.0)
}

fn phong_specular(
    normal: Vec3,
    light_direction: Vec3,
    view_direction: Vec3,
    material_specular: f32,
    light_intensity: f32,
    shininess: f32,
) -> f32 {
    if material_specular == 0.0 {
        return 0.0;
    }

    let reflected_light = normal * (2.0 * normal.dot(light_direction)) - light_direction;
    reflected_light
        .normalize()
        .dot(view_direction.normalize())
        .max(0.0)
        .powf(shininess)
        * material_specular
        * light_intensity
}

fn is_shadowed(
    shadow_origin: &Vec3,
    light_direction: &Vec3,
    light_distance: f32,
    cubes: &[Cube],
) -> bool {
    cubes.iter().any(|cube| {
        let hit = cube.ray_intersect(shadow_origin, light_direction);
        hit.is_intersecting && hit.distance < light_distance
    })
}

fn color_to_vec3(color: Color) -> Vec3 {
    Vec3::new(
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use raylib::prelude::Color;

    use super::*;
    use crate::material::Material;

    #[test]
    fn lambert_clamps_surfaces_facing_away_from_light() {
        let normal = Vec3::new(0.0, 1.0, 0.0);

        assert_eq!(lambert(normal, Vec3::new(0.0, -1.0, 0.0)), 0.0);
        assert_eq!(lambert(normal, Vec3::new(0.0, 1.0, 0.0)), 1.0);
    }

    #[test]
    fn zero_material_specular_produces_no_highlight() {
        let direction = Vec3::new(0.0, 0.0, 1.0);

        assert_eq!(
            phong_specular(direction, direction, direction, 0.0, 1.0, 32.0),
            0.0
        );
    }

    #[test]
    fn only_geometry_before_the_light_casts_a_shadow() {
        let material = Material::new(Color::WHITE);
        let origin = Vec3::default();
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let before_light = [Cube::from_center_size(
            Vec3::new(0.0, 0.0, 5.0),
            1.0,
            material,
        )];
        let behind_light = [Cube::from_center_size(
            Vec3::new(0.0, 0.0, 15.0),
            1.0,
            material,
        )];

        assert!(is_shadowed(&origin, &direction, 10.0, &before_light));
        assert!(!is_shadowed(&origin, &direction, 10.0, &behind_light));
    }

    #[test]
    fn normal_bias_avoids_immediate_self_intersection() {
        let cube = Cube::from_center_size(Vec3::default(), 1.0, Material::new(Color::WHITE));
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let surface_point = Vec3::new(0.0, 0.0, 0.5);
        let biased_origin = surface_point + normal * 0.001;

        assert!(!is_shadowed(&biased_origin, &normal, 10.0, &[cube]));
    }
}
