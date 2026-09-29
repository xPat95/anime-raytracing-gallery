use std::thread;

use raylib::prelude::Color;

use crate::{
    bvh::Bvh, camera::Camera, cube::Cube, framebuffer::Framebuffer, ground::GroundPlane,
    intersect::Intersect, light::LightingConfig, ray_intersect::RayIntersect, sky::SkyType,
    texture::Texture, vec3::Vec3,
};

const FRAMEBUFFER_CLEAR: Color = Color::new(18, 22, 30, 255);
const MAX_RAY_DEPTH: u32 = 8;
const MAX_SHADOW_HITS: u32 = 16;
const TRANSMISSION_BIAS: f32 = 0.001;
const REFLECTION_BIAS: f32 = 0.001;
const MIN_RAY_CONTRIBUTION: f32 = 0.001;

#[derive(Clone, Copy, Debug)]
pub enum RenderStrategy {
    LinearSingleThread,
    BvhSingleThread,
    BvhMultiThread,
}

#[derive(Clone, Copy)]
enum Accelerator<'a> {
    Linear(&'a [Cube]),
    Bvh(&'a Bvh, &'a [Cube]),
}

#[derive(Clone, Copy)]
struct SceneHit {
    intersection: Intersect,
    is_ground: bool,
}

#[derive(Clone, Copy)]
struct TextureSnapshot<'a> {
    textures: &'a [Texture],
    frames: &'a [usize],
}

impl Accelerator<'_> {
    fn closest_hit(self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        match self {
            Self::Linear(cubes) => cast_ray_linear(ray_origin, ray_direction, cubes),
            Self::Bvh(bvh, cubes) => bvh.closest_hit(ray_origin, ray_direction, cubes),
        }
    }

    fn closest_hit_with_ground(
        self,
        ray_origin: &Vec3,
        ray_direction: &Vec3,
        ground: &GroundPlane,
    ) -> SceneHit {
        let geometry_hit = self.closest_hit(ray_origin, ray_direction);
        let ground_hit = ground.ray_intersect(ray_origin, ray_direction);
        if ground_hit.is_intersecting && ground_hit.distance < geometry_hit.distance {
            SceneHit {
                intersection: ground_hit,
                is_ground: true,
            }
        } else {
            SceneHit {
                intersection: geometry_hit,
                is_ground: false,
            }
        }
    }

    fn is_occluded(self, ray_origin: &Vec3, ray_direction: &Vec3, max_distance: f32) -> bool {
        match self {
            Self::Linear(cubes) => cubes.iter().any(|cube| {
                let hit = cube.ray_intersect(ray_origin, ray_direction);
                hit.is_intersecting && hit.distance < max_distance
            }),
            Self::Bvh(bvh, cubes) => {
                bvh.is_occluded(ray_origin, ray_direction, max_distance, cubes)
            }
        }
    }
}

pub fn render(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    cubes: &[Cube],
    bvh: &Bvh,
    textures: &[Texture],
    lighting: &LightingConfig,
    sky: SkyType,
    ground: &GroundPlane,
    render_time_seconds: f32,
    strategy: RenderStrategy,
) -> usize {
    let texture_frames: Vec<usize> = textures
        .iter()
        .map(|texture| texture.frame_index(render_time_seconds))
        .collect();
    let texture_snapshot = TextureSnapshot {
        textures,
        frames: &texture_frames,
    };
    let accelerator = match strategy {
        RenderStrategy::LinearSingleThread => Accelerator::Linear(cubes),
        RenderStrategy::BvhSingleThread | RenderStrategy::BvhMultiThread => {
            Accelerator::Bvh(bvh, cubes)
        }
    };
    let worker_count = match strategy {
        RenderStrategy::BvhMultiThread => thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
            .min(framebuffer.height() as usize),
        _ => 1,
    };
    let pixels = if worker_count == 1 {
        render_rows(
            0,
            framebuffer.height(),
            framebuffer.width(),
            framebuffer.height(),
            camera,
            accelerator,
            texture_snapshot,
            lighting,
            sky,
            ground,
        )
    } else {
        render_parallel(
            framebuffer.width(),
            framebuffer.height(),
            worker_count,
            camera,
            accelerator,
            texture_snapshot,
            lighting,
            sky,
            ground,
        )
    };
    framebuffer
        .replace_pixels(pixels)
        .expect("render produced an invalid number of pixels");
    worker_count
}

fn render_parallel(
    width: u32,
    height: u32,
    worker_count: usize,
    camera: &Camera,
    accelerator: Accelerator<'_>,
    texture_snapshot: TextureSnapshot<'_>,
    lighting: &LightingConfig,
    sky: SkyType,
    ground: &GroundPlane,
) -> Vec<Color> {
    let rows_per_worker = (height as usize).div_ceil(worker_count);
    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(worker_count);
        for worker in 0..worker_count {
            let start_y = (worker * rows_per_worker) as u32;
            let end_y = ((worker + 1) * rows_per_worker).min(height as usize) as u32;
            if start_y >= end_y {
                continue;
            }
            handles.push((
                start_y,
                scope.spawn(move || {
                    render_rows(
                        start_y,
                        end_y,
                        width,
                        height,
                        camera,
                        accelerator,
                        texture_snapshot,
                        lighting,
                        sky,
                        ground,
                    )
                }),
            ));
        }

        let mut pixels = vec![FRAMEBUFFER_CLEAR; (width * height) as usize];
        for (start_y, handle) in handles {
            let rows = handle.join().expect("render worker panicked");
            let start = (start_y * width) as usize;
            pixels[start..start + rows.len()].copy_from_slice(&rows);
        }
        pixels
    })
}

#[allow(clippy::too_many_arguments)]
fn render_rows(
    start_y: u32,
    end_y: u32,
    width: u32,
    height: u32,
    camera: &Camera,
    accelerator: Accelerator<'_>,
    texture_snapshot: TextureSnapshot<'_>,
    lighting: &LightingConfig,
    sky: SkyType,
    ground: &GroundPlane,
) -> Vec<Color> {
    let mut pixels = Vec::with_capacity(((end_y - start_y) * width) as usize);
    for y in start_y..end_y {
        for x in 0..width {
            let ray_origin = camera.position();
            let ray_direction = camera.ray_direction(x, y, width, height);
            let color = trace_ray(
                &ray_origin,
                &ray_direction,
                accelerator,
                texture_snapshot,
                lighting,
                sky,
                ground,
                0,
                1.0,
            );
            pixels.push(vec3_to_color(color));
        }
    }
    pixels
}

fn cast_ray_linear(ray_origin: &Vec3, ray_direction: &Vec3, cubes: &[Cube]) -> Intersect {
    let mut closest_hit = Intersect::empty();

    for cube in cubes {
        let hit = cube.ray_intersect(ray_origin, ray_direction);

        if hit.is_intersecting && hit.distance < closest_hit.distance {
            closest_hit = hit;
        }
    }

    closest_hit
}

fn trace_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    accelerator: Accelerator<'_>,
    texture_snapshot: TextureSnapshot<'_>,
    lighting: &LightingConfig,
    sky: SkyType,
    ground: &GroundPlane,
    depth: u32,
    contribution: f32,
) -> Vec3 {
    let scene_hit = accelerator.closest_hit_with_ground(ray_origin, ray_direction, ground);
    let hit = scene_hit.intersection;
    if !hit.is_intersecting {
        return background_color(*ray_direction, sky, visible_sun_direction(sky, lighting));
    }

    let hit_point = *ray_origin + *ray_direction * hit.distance;
    let geometric_normal = hit.normal.normalize();
    let shading_normal = if scene_hit.is_ground {
        ground.shading_normal(hit_point)
    } else {
        geometric_normal
    };

    let (surface, tint) = shade_surface(
        hit,
        hit_point,
        ray_direction,
        shading_normal,
        geometric_normal,
        scene_hit.is_ground,
        accelerator,
        texture_snapshot,
        lighting,
        ground,
    );
    let transparency = hit.material.transparency.clamp(0.0, 1.0);
    let reflectivity = hit.material.reflectivity.clamp(0.0, 1.0);
    let transmission_weight = (1.0 - reflectivity) * transparency;
    let transmission_blend = if can_transmit(transparency, depth)
        && is_significant(contribution * transmission_weight)
    {
        let transmitted_direction =
            transmission_direction(*ray_direction, shading_normal, hit.material.ior);
        let transmitted_origin = secondary_ray_origin(
            hit_point,
            geometric_normal,
            transmitted_direction,
            TRANSMISSION_BIAS,
        );
        let transmitted = trace_ray(
            &transmitted_origin,
            &transmitted_direction,
            accelerator,
            texture_snapshot,
            lighting,
            sky,
            ground,
            depth + 1,
            contribution * transmission_weight,
        );
        blend_transparency(surface, transmitted, tint, transparency)
    } else {
        surface
    };

    if !can_reflect(reflectivity, depth) || !is_significant(contribution * reflectivity) {
        return transmission_blend;
    }

    let reflected_direction = reflect(*ray_direction, shading_normal).normalize();
    let reflected_origin = secondary_ray_origin(
        hit_point,
        geometric_normal,
        reflected_direction,
        REFLECTION_BIAS,
    );
    let reflected = trace_ray(
        &reflected_origin,
        &reflected_direction,
        accelerator,
        texture_snapshot,
        lighting,
        sky,
        ground,
        depth + 1,
        contribution * reflectivity,
    );
    blend_reflection(transmission_blend, reflected, reflectivity)
}

fn can_transmit(transparency: f32, depth: u32) -> bool {
    transparency > 0.0 && depth < MAX_RAY_DEPTH
}

fn can_reflect(reflectivity: f32, depth: u32) -> bool {
    reflectivity > 0.0 && depth < MAX_RAY_DEPTH
}

fn is_significant(contribution: f32) -> bool {
    contribution >= MIN_RAY_CONTRIBUTION
}

fn shade_surface(
    hit: Intersect,
    hit_point: Vec3,
    ray_direction: &Vec3,
    normal: Vec3,
    geometric_normal: Vec3,
    is_ground: bool,
    accelerator: Accelerator<'_>,
    texture_snapshot: TextureSnapshot<'_>,
    lighting: &LightingConfig,
    ground: &GroundPlane,
) -> (Vec3, Vec3) {
    let material_tint = color_to_vec3(hit.material.base_color);
    let mut base_color = hit
        .material
        .texture_index
        .and_then(|index| {
            texture_snapshot
                .textures
                .get(index)
                .map(|texture| (index, texture))
        })
        .map_or(material_tint, |(index, texture)| {
            component_multiply(
                color_to_vec3(texture.sample_frame(hit.u, hit.v, texture_snapshot.frames[index])),
                material_tint,
            )
        });
    if is_ground {
        base_color = ground.surface_color(hit_point, base_color);
    }
    let to_light = lighting.light.position - hit_point;
    let light_distance = to_light.length();
    let light_direction = to_light / light_distance;
    let diffuse_angle = lambert(normal, light_direction);
    let shadow_origin = hit_point + geometric_normal * lighting.shadow_bias;
    let light_visibility = if diffuse_angle > 0.0 {
        shadow_visibility(
            &shadow_origin,
            &light_direction,
            light_distance,
            accelerator,
            ground,
        )
    } else {
        1.0
    };

    let diffuse = diffuse_angle * hit.material.albedo * lighting.light.intensity * light_visibility;
    let specular = if diffuse_angle == 0.0 {
        0.0
    } else {
        phong_specular(
            normal,
            light_direction,
            -*ray_direction,
            hit.material.specular,
            lighting.light.intensity,
            lighting.phong_shininess,
        ) * light_visibility
    };
    let light_color = color_to_vec3(lighting.light.color);
    let lit_color = Vec3::new(
        base_color.x * (lighting.ambient_intensity + diffuse * light_color.x)
            + specular * light_color.x,
        base_color.y * (lighting.ambient_intensity + diffuse * light_color.y)
            + specular * light_color.y,
        base_color.z * (lighting.ambient_intensity + diffuse * light_color.z)
            + specular * light_color.z,
    );
    let emission = emission_contribution(base_color, hit.material);

    (lit_color + emission, transmission_tint(base_color))
}

fn emission_contribution(base_color: Vec3, material: crate::material::Material) -> Vec3 {
    if material.emission_strength == 0.0 {
        return Vec3::default();
    }
    component_multiply(base_color, color_to_vec3(material.emission_tint))
        * material.emission_strength
}

fn blend_transparency(surface: Vec3, transmitted: Vec3, tint: Vec3, transparency: f32) -> Vec3 {
    surface * (1.0 - transparency) + component_multiply(transmitted, tint) * transparency
}

fn blend_reflection(base: Vec3, reflected: Vec3, reflectivity: f32) -> Vec3 {
    base * (1.0 - reflectivity) + reflected * reflectivity
}

fn transmission_tint(surface_color: Vec3) -> Vec3 {
    Vec3::new(
        0.6 + surface_color.x * 0.4,
        0.6 + surface_color.y * 0.4,
        0.6 + surface_color.z * 0.4,
    )
}

fn secondary_ray_origin(
    hit_point: Vec3,
    geometric_normal: Vec3,
    direction: Vec3,
    bias: f32,
) -> Vec3 {
    let bias_normal = if direction.dot(geometric_normal) >= 0.0 {
        geometric_normal
    } else {
        -geometric_normal
    };
    hit_point + bias_normal * bias
}

fn transmission_direction(incident: Vec3, outward_normal: Vec3, ior: Option<f32>) -> Vec3 {
    let incident = incident.normalize();
    let Some(material_ior) = ior else {
        return incident;
    };
    let entering = incident.dot(outward_normal) < 0.0;
    let (normal, eta_ratio) = if entering {
        (outward_normal, 1.0 / material_ior)
    } else {
        (-outward_normal, material_ior)
    };

    refract(incident, normal, eta_ratio)
        .unwrap_or_else(|| reflect(incident, normal))
        .normalize()
}

fn refract(incident: Vec3, oriented_normal: Vec3, eta_ratio: f32) -> Option<Vec3> {
    let cos_theta = (-incident).dot(oriented_normal).min(1.0);
    let perpendicular = (incident + oriented_normal * cos_theta) * eta_ratio;
    let parallel_length_squared = 1.0 - perpendicular.dot(perpendicular);
    if parallel_length_squared < 0.0 {
        return None;
    }
    Some(perpendicular - oriented_normal * parallel_length_squared.sqrt())
}

fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - normal * (2.0 * incident.dot(normal))
}

fn background_color(ray_direction: Vec3, sky: SkyType, sun_direction: Option<Vec3>) -> Vec3 {
    crate::sky::sample_sky(ray_direction, sky, sun_direction)
}

fn visible_sun_direction(sky: SkyType, lighting: &LightingConfig) -> Option<Vec3> {
    matches!(sky, SkyType::Sunset | SkyType::ClearDay).then(|| lighting.light.position.normalize())
}

fn shadow_visibility(
    shadow_origin: &Vec3,
    light_direction: &Vec3,
    light_distance: f32,
    accelerator: Accelerator<'_>,
    ground: &GroundPlane,
) -> f32 {
    let mut visibility = 1.0;
    let mut origin = *shadow_origin;
    let mut remaining_distance = light_distance;

    for _ in 0..MAX_SHADOW_HITS {
        let hit = accelerator
            .closest_hit_with_ground(&origin, light_direction, ground)
            .intersection;
        if !hit.is_intersecting || hit.distance >= remaining_distance {
            break;
        }
        visibility *= hit.material.transparency.clamp(0.0, 1.0);
        if visibility <= f32::EPSILON {
            debug_assert!(accelerator.is_occluded(&origin, light_direction, remaining_distance));
            return 0.0;
        }
        let advance = hit.distance + TRANSMISSION_BIAS;
        origin = origin + *light_direction * advance;
        remaining_distance -= advance;
        if remaining_distance <= 0.0 {
            break;
        }
    }

    visibility
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

fn color_to_vec3(color: Color) -> Vec3 {
    Vec3::new(
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
    )
}

fn vec3_to_color(color: Vec3) -> Color {
    Color::new(
        (color.x.clamp(0.0, 1.0) * 255.0) as u8,
        (color.y.clamp(0.0, 1.0) * 255.0) as u8,
        (color.z.clamp(0.0, 1.0) * 255.0) as u8,
        255,
    )
}

fn component_multiply(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x * right.x, left.y * right.y, left.z * right.z)
}

#[cfg(test)]
mod tests {
    use raylib::prelude::Color;

    use super::*;
    use crate::{
        camera::CameraConfig,
        ground::{GroundKind, GroundPlane, SolidGroundStyle},
        light::PointLight,
        material::Material,
    };

    fn distant_ground() -> GroundPlane {
        GroundPlane::new(
            GroundKind::Solid,
            Some(SolidGroundStyle::Stone),
            -1_000.0,
            Material::new(Color::WHITE),
        )
    }

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
    fn zero_emission_adds_nothing() {
        let material = Material::new(Color::WHITE);
        assert_eq!(
            emission_contribution(Vec3::new(0.4, 0.2, 0.8), material),
            Vec3::default()
        );
    }

    #[test]
    fn emission_preserves_surface_modulation_without_lighting() {
        let mut material = Material::new(Color::WHITE);
        material.emission_tint = Color::new(128, 255, 64, 255);
        material.emission_strength = 0.5;
        let dark_texel = emission_contribution(Vec3::new(0.2, 0.1, 0.4), material);
        let bright_texel = emission_contribution(Vec3::new(0.8, 0.4, 0.9), material);

        assert!(bright_texel.x > dark_texel.x);
        assert!(bright_texel.y > dark_texel.y);
        assert!(bright_texel.z > dark_texel.z);
        assert_eq!(
            dark_texel,
            emission_contribution(Vec3::new(0.2, 0.1, 0.4), material)
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

        assert_eq!(
            shadow_visibility(
                &origin,
                &direction,
                10.0,
                Accelerator::Linear(&before_light),
                &distant_ground()
            ),
            0.0
        );
        assert_eq!(
            shadow_visibility(
                &origin,
                &direction,
                10.0,
                Accelerator::Linear(&behind_light),
                &distant_ground()
            ),
            1.0
        );
    }

    #[test]
    fn normal_bias_avoids_immediate_self_intersection() {
        let cube = Cube::from_center_size(Vec3::default(), 1.0, Material::new(Color::WHITE));
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let surface_point = Vec3::new(0.0, 0.0, 0.5);
        let biased_origin = surface_point + normal * 0.001;

        assert_eq!(
            shadow_visibility(
                &biased_origin,
                &normal,
                10.0,
                Accelerator::Linear(&[cube]),
                &distant_ground(),
            ),
            1.0
        );
    }

    #[test]
    fn transparency_blends_surface_and_tinted_transmission() {
        let surface = Vec3::new(1.0, 0.0, 0.0);
        let transmitted = Vec3::new(0.0, 0.0, 1.0);
        let neutral_tint = Vec3::new(1.0, 1.0, 1.0);

        assert_eq!(
            blend_transparency(surface, transmitted, neutral_tint, 0.0),
            surface
        );
        assert_eq!(
            blend_transparency(surface, transmitted, neutral_tint, 1.0),
            transmitted
        );
        assert_eq!(
            blend_transparency(surface, transmitted, neutral_tint, 0.5),
            Vec3::new(0.5, 0.0, 0.5)
        );
    }

    #[test]
    fn reflection_blend_respects_zero_full_and_intermediate_values() {
        let base = Vec3::new(1.0, 0.0, 0.0);
        let reflected = Vec3::new(0.0, 0.0, 1.0);

        assert_eq!(blend_reflection(base, reflected, 0.0), base);
        assert_eq!(blend_reflection(base, reflected, 1.0), reflected);
        assert_eq!(
            blend_reflection(base, reflected, 0.25),
            Vec3::new(0.75, 0.0, 0.25)
        );
    }

    #[test]
    fn perpendicular_reflection_reverses_direction() {
        let incident = Vec3::new(0.0, 0.0, -1.0);
        let reflected = reflect(incident, Vec3::new(0.0, 0.0, 1.0));

        assert_eq!(reflected, Vec3::new(0.0, 0.0, 1.0));
        assert!((reflected.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn forty_five_degree_reflection_preserves_angle() {
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let incident = Vec3::new(1.0, 0.0, -1.0).normalize();
        let reflected = reflect(incident, normal).normalize();

        assert!(((-incident).dot(normal) - reflected.dot(normal)).abs() < 0.0001);
        assert!((reflected.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn secondary_ray_origin_uses_geometric_output_side() {
        let hit_point = Vec3::new(1.0, 2.0, 3.0);
        let normal = Vec3::new(0.0, 0.0, 1.0);

        let outside = secondary_ray_origin(hit_point, normal, normal, REFLECTION_BIAS);
        let inside = secondary_ray_origin(hit_point, normal, -normal, REFLECTION_BIAS);

        assert!((outside.z - 3.001).abs() < 0.00001);
        assert!((inside.z - 2.999).abs() < 0.00001);
    }

    #[test]
    fn reflection_and_transmission_share_maximum_depth() {
        assert!(can_reflect(0.5, MAX_RAY_DEPTH - 1));
        assert!(!can_reflect(0.5, MAX_RAY_DEPTH));
        assert!(!can_reflect(0.0, 0));
        assert!(!can_transmit(0.5, MAX_RAY_DEPTH));
    }

    #[test]
    fn negligible_secondary_contributions_are_skipped() {
        assert!(is_significant(MIN_RAY_CONTRIBUTION));
        assert!(!is_significant(MIN_RAY_CONTRIBUTION * 0.5));
    }

    #[test]
    fn refractive_material_can_also_be_reflective() {
        let mut material = Material::new(Color::WHITE);
        material.transparency = 0.35;
        material.reflectivity = 0.12;
        material.ior = Some(1.5);

        assert!(can_transmit(material.transparency, 0));
        assert!(can_reflect(material.reflectivity, 0));
        assert_eq!(material.ior, Some(1.5));
    }

    #[test]
    fn every_ray_miss_uses_the_shared_environment_sampler() {
        let direction = Vec3::new(0.2, 0.4, -0.8).normalize();

        assert_eq!(
            background_color(direction, SkyType::AuroraNight, None),
            crate::sky::sample_sky(direction, SkyType::AuroraNight, None)
        );
    }

    #[test]
    fn transmitted_ray_keeps_direction_and_starts_after_surface() {
        let hit_point = Vec3::new(1.0, 2.0, 3.0);
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let origin = secondary_ray_origin(hit_point, direction, direction, TRANSMISSION_BIAS);

        assert!((origin.z - 3.001).abs() < 0.00001);
        assert_eq!((origin - hit_point).normalize(), direction);
    }

    #[test]
    fn transparent_shadow_attenuates_while_opaque_shadow_blocks() {
        let mut transparent_material = Material::new(Color::WHITE);
        transparent_material.transparency = 0.5;
        let transparent_cube = [Cube::from_center_size(
            Vec3::new(0.0, 0.0, 5.0),
            1.0,
            transparent_material,
        )];
        let opaque_cube = [Cube::from_center_size(
            Vec3::new(0.0, 0.0, 5.0),
            1.0,
            Material::new(Color::WHITE),
        )];
        let origin = Vec3::default();
        let direction = Vec3::new(0.0, 0.0, 1.0);

        let transparent_visibility = shadow_visibility(
            &origin,
            &direction,
            10.0,
            Accelerator::Linear(&transparent_cube),
            &distant_ground(),
        );
        assert!(transparent_visibility > 0.0 && transparent_visibility < 1.0);
        assert_eq!(
            shadow_visibility(
                &origin,
                &direction,
                10.0,
                Accelerator::Linear(&opaque_cube),
                &distant_ground(),
            ),
            0.0
        );
    }

    #[test]
    fn maximum_depth_stops_transmission() {
        assert!(can_transmit(0.5, MAX_RAY_DEPTH - 1));
        assert!(!can_transmit(0.5, MAX_RAY_DEPTH));
        assert!(!can_transmit(0.0, 0));
    }

    #[test]
    fn perpendicular_air_to_glass_keeps_direction() {
        let incident = Vec3::new(0.0, 0.0, -1.0);
        let direction = transmission_direction(incident, Vec3::new(0.0, 0.0, 1.0), Some(1.5));

        assert!((direction - incident).length() < 0.0001);
        assert!((direction.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn air_to_glass_bends_toward_normal() {
        let incident = Vec3::new(1.0, 0.0, -1.0).normalize();
        let direction = transmission_direction(incident, Vec3::new(0.0, 0.0, 1.0), Some(1.5));

        assert!(direction.x.abs() < incident.x.abs());
        assert!(direction.z.abs() > incident.z.abs());
    }

    #[test]
    fn glass_to_air_bends_away_from_normal() {
        let incident = Vec3::new(0.3, 0.0, 0.9539392).normalize();
        let direction = transmission_direction(incident, Vec3::new(0.0, 0.0, 1.0), Some(1.5));

        assert!(direction.x.abs() > incident.x.abs());
        assert!(direction.z.abs() < incident.z.abs());
    }

    #[test]
    fn total_internal_reflection_is_detected_and_reflected() {
        let incident = Vec3::new(0.9, 0.0, 0.4358899).normalize();
        let oriented_normal = Vec3::new(0.0, 0.0, -1.0);

        assert!(refract(incident, oriented_normal, 1.5).is_none());
        let direction = transmission_direction(incident, -oriented_normal, Some(1.5));
        assert!(direction.z < 0.0);
        assert!((direction.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn transparent_material_without_ior_transmits_straight() {
        let incident = Vec3::new(0.4, 0.0, -1.0).normalize();

        assert_eq!(
            transmission_direction(incident, Vec3::new(0.0, 0.0, 1.0), None),
            incident
        );
    }

    #[test]
    fn single_and_multithread_bvh_framebuffers_match() {
        let cubes = [Cube::from_center_size(
            Vec3::default(),
            20.0,
            Material::new(Color::WHITE),
        )];
        let bvh = Bvh::build(&cubes);
        let camera = Camera::new(CameraConfig::from_position(
            Vec3::default(),
            Vec3::new(0.0, 0.0, 30.0),
            1.0,
            100.0,
        ));
        let lighting = LightingConfig {
            light: PointLight {
                position: Vec3::new(-10.0, 20.0, 20.0),
                color: Color::WHITE,
                intensity: 1.0,
            },
            ambient_intensity: 0.1,
            shadow_bias: 0.001,
            phong_shininess: 32.0,
        };
        let mut single = Framebuffer::new(32, 18, Color::BLACK);
        let mut multi = Framebuffer::new(32, 18, Color::BLACK);
        let ground = distant_ground();

        render(
            &mut single,
            &camera,
            &cubes,
            &bvh,
            &[],
            &lighting,
            SkyType::ClearDay,
            &ground,
            0.0,
            RenderStrategy::BvhSingleThread,
        );
        render(
            &mut multi,
            &camera,
            &cubes,
            &bvh,
            &[],
            &lighting,
            SkyType::ClearDay,
            &ground,
            0.0,
            RenderStrategy::BvhMultiThread,
        );

        let difference = single.difference(&multi).unwrap();
        assert_eq!(difference.different_pixels, 0);
        assert_eq!(difference.maximum_channel_difference, 0);
    }

    #[test]
    fn visible_sun_is_derived_from_the_existing_point_light() {
        let lighting = LightingConfig {
            light: PointLight {
                position: Vec3::new(-90.0, 125.0, 140.0),
                color: Color::WHITE,
                intensity: 1.15,
            },
            ambient_intensity: 0.1,
            shadow_bias: 0.001,
            phong_shininess: 32.0,
        };
        assert_eq!(
            visible_sun_direction(SkyType::Sunset, &lighting),
            Some(lighting.light.position.normalize())
        );
        assert_eq!(
            visible_sun_direction(SkyType::ClearDay, &lighting),
            Some(lighting.light.position.normalize())
        );
        assert_eq!(visible_sun_direction(SkyType::Cloudy, &lighting), None);
        assert_eq!(visible_sun_direction(SkyType::StarryNight, &lighting), None);
        assert_eq!(visible_sun_direction(SkyType::AuroraNight, &lighting), None);
    }
}
