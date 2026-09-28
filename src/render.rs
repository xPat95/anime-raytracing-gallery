use raylib::prelude::Color;

use crate::{
    camera::Camera, cube::Cube, framebuffer::Framebuffer, intersect::Intersect,
    ray_intersect::RayIntersect, vec3::Vec3,
};

const BACKGROUND: Color = Color::new(18, 22, 30, 255);

pub fn render(framebuffer: &mut Framebuffer, camera: &Camera, cubes: &[Cube]) {
    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            let ray_origin = camera.position();
            let ray_direction =
                camera.ray_direction(x, y, framebuffer.width(), framebuffer.height());
            let closest_hit = cast_ray(&ray_origin, &ray_direction, cubes);
            let color = match closest_hit.is_intersecting {
                true => shade_by_normal(closest_hit),
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

fn shade_by_normal(hit: Intersect) -> Color {
    let normal_light =
        (hit.normal.z.abs() * 0.30 + hit.normal.x.abs() * 0.18 + hit.normal.y.abs() * 0.10)
            .clamp(0.10, 0.35);
    let factor = 0.70 + normal_light;

    Color::new(
        (hit.material.albedo.r as f32 * factor).min(255.0) as u8,
        (hit.material.albedo.g as f32 * factor).min(255.0) as u8,
        (hit.material.albedo.b as f32 * factor).min(255.0) as u8,
        255,
    )
}
