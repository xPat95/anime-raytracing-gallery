use raylib::prelude::Color;

use crate::{
    camera::Camera, cube::Cube, framebuffer::Framebuffer, intersect::Intersect,
    ray_intersect::RayIntersect, texture::Texture, vec3::Vec3,
};

const BACKGROUND: Color = Color::new(18, 22, 30, 255);

pub fn render(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    cubes: &[Cube],
    textures: &[Texture],
) {
    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            let ray_origin = camera.position();
            let ray_direction =
                camera.ray_direction(x, y, framebuffer.width(), framebuffer.height());
            let closest_hit = cast_ray(&ray_origin, &ray_direction, cubes);
            let color = match closest_hit.is_intersecting {
                true => shade_hit(closest_hit, textures),
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

fn shade_hit(hit: Intersect, textures: &[Texture]) -> Color {
    let surface_color = hit
        .material
        .texture_index
        .and_then(|index| textures.get(index))
        .map_or(hit.material.albedo, |texture| texture.sample(hit.u, hit.v));
    let normal_light =
        (hit.normal.z.abs() * 0.30 + hit.normal.x.abs() * 0.18 + hit.normal.y.abs() * 0.10)
            .clamp(0.10, 0.35);
    let factor = 0.70 + normal_light;

    Color::new(
        (surface_color.r as f32 * factor).min(255.0) as u8,
        (surface_color.g as f32 * factor).min(255.0) as u8,
        (surface_color.b as f32 * factor).min(255.0) as u8,
        surface_color.a,
    )
}
