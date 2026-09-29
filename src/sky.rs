use std::f32::consts::{PI, TAU};

use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyType {
    Sunset,
    Cloudy,
    StarryNight,
    ClearDay,
    AuroraNight,
}

pub fn sample_sky(direction: Vec3, sky: SkyType) -> Vec3 {
    let direction = direction.normalize();
    let color = match sky {
        SkyType::Sunset => sunset(direction),
        SkyType::Cloudy => cloudy(direction),
        SkyType::StarryNight => starry_night(direction),
        SkyType::ClearDay => clear_day(direction),
        SkyType::AuroraNight => aurora_night(direction),
    };
    clamp_color(color)
}

fn sunset(direction: Vec3) -> Vec3 {
    let horizon = Vec3::new(0.95, 0.38, 0.12);
    let middle = Vec3::new(0.58, 0.16, 0.28);
    let zenith = Vec3::new(0.10, 0.08, 0.24);
    let lower = Vec3::new(0.12, 0.09, 0.15);
    let lower_to_horizon = atmospheric_gradient(lower, horizon, middle, direction.y);
    let color = mix(
        lower_to_horizon,
        zenith,
        smoothstep(0.25, 0.90, direction.y),
    );
    add_sun(
        color,
        direction,
        Vec3::new(-0.72, -0.10, -0.68).normalize(),
        Vec3::new(1.0, 0.94, 0.72),
        Vec3::new(1.0, 0.43, 0.10),
        0.99955,
        0.965,
        0.22,
    )
}

fn cloudy(direction: Vec3) -> Vec3 {
    let horizon = Vec3::new(0.42, 0.48, 0.55);
    let zenith = Vec3::new(0.14, 0.19, 0.27);
    let lower = Vec3::new(0.10, 0.13, 0.17);
    let sky = atmospheric_gradient(lower, horizon, zenith, direction.y);
    let longitude = direction.z.atan2(direction.x);
    let low = (longitude * 1.35 + direction.y * 2.7 + direction.z * 0.8).sin();
    let medium = (longitude * 2.8 - direction.y * 4.1 + direction.x * 1.7).cos();
    let irregular = (longitude * 4.9 + direction.y * 3.3 + low * 0.9).sin();
    let detail = (longitude * 7.3 - direction.y * 5.6 + direction.z * 2.1).cos();
    let mass = low * 0.48 + medium * 0.29 + irregular * 0.17 + detail * 0.06;
    let cloud = smoothstep(-0.52, 0.38, mass);
    let cloud_light = smoothstep(-0.30, 0.62, mass + medium * 0.16);
    let cloud_color = mix(
        Vec3::new(0.15, 0.19, 0.26),
        Vec3::new(0.66, 0.69, 0.70),
        cloud_light,
    );
    mix(sky, cloud_color, 0.48 + cloud * 0.34)
}

fn starry_night(direction: Vec3) -> Vec3 {
    let base = night_gradient(direction, Vec3::new(0.055, 0.10, 0.22));
    let stars = star_field(direction) * smoothstep(-0.55, -0.08, direction.y);
    let with_stars = base + Vec3::new(0.78, 0.86, 1.0) * stars;
    add_moon(
        with_stars,
        direction,
        Vec3::new(-0.45, -0.12, -0.88).normalize(),
        Vec3::new(0.82, 0.88, 1.0),
        0.9995,
        0.996,
        0.08,
    )
}

fn clear_day(direction: Vec3) -> Vec3 {
    let horizon = Vec3::new(0.72, 0.88, 1.0);
    let zenith = Vec3::new(0.08, 0.40, 0.88);
    let lower = Vec3::new(0.30, 0.48, 0.65);
    let color = atmospheric_gradient(lower, horizon, zenith, direction.y);
    add_sun(
        color,
        direction,
        Vec3::new(-0.62, -0.18, -0.76).normalize(),
        Vec3::new(1.0, 0.98, 0.86),
        Vec3::new(1.0, 0.88, 0.55),
        0.99965,
        0.970,
        0.16,
    )
}

fn aurora_night(direction: Vec3) -> Vec3 {
    let base = night_gradient(direction, Vec3::new(0.035, 0.11, 0.20));
    let stars = star_field(direction) * smoothstep(-0.58, -0.10, direction.y) * 0.65;
    let longitude = direction.z.atan2(direction.x);
    let height_mask =
        smoothstep(-0.72, -0.16, direction.y) * (1.0 - smoothstep(0.34, 0.82, direction.y));
    let wave = (longitude * 2.1 + (longitude * 5.3).sin() * 0.38).sin();
    let ribbon_center = -0.16 + wave * 0.14 + (longitude * 3.7).cos() * 0.05;
    let ribbon = 1.0 - smoothstep(0.035, 0.22, (direction.y - ribbon_center).abs());
    let curtains = 0.55 + 0.45 * (longitude * 11.0 + direction.y * 7.0).sin().abs();
    let intensity = ribbon * curtains * height_mask * 0.72;
    let aurora_color = mix(
        Vec3::new(0.10, 0.95, 0.55),
        Vec3::new(0.08, 0.72, 0.92),
        smoothstep(0.12, 0.62, direction.y),
    );
    base + Vec3::new(0.70, 0.82, 1.0) * stars + aurora_color * intensity
}

fn night_gradient(direction: Vec3, horizon: Vec3) -> Vec3 {
    let zenith = Vec3::new(0.008, 0.016, 0.055);
    let lower = Vec3::new(0.012, 0.025, 0.055);
    atmospheric_gradient(lower, horizon, zenith, direction.y)
}

fn star_field(direction: Vec3) -> f32 {
    let u = direction.z.atan2(direction.x) / TAU + 0.5;
    let v = direction.y.clamp(-1.0, 1.0).asin() / PI + 0.5;
    let cell_x = (u * 720.0).floor();
    let cell_y = (v * 360.0).floor();
    let local_x = (u * 720.0).fract();
    let local_y = (v * 360.0).fract();
    let density = hash(cell_x, cell_y);
    if density < 0.965 {
        return 0.0;
    }
    let center_x = 0.2 + hash(cell_x + 17.0, cell_y + 31.0) * 0.6;
    let center_y = 0.2 + hash(cell_x + 47.0, cell_y + 11.0) * 0.6;
    let distance = ((local_x - center_x).powi(2) + (local_y - center_y).powi(2)).sqrt();
    (1.0 - smoothstep(0.025, 0.095, distance)) * (0.55 + density * 0.45)
}

fn hash(x: f32, y: f32) -> f32 {
    ((x * 127.1 + y * 311.7).sin() * 43_758.547).fract().abs()
}

fn add_sun(
    base: Vec3,
    direction: Vec3,
    sun_direction: Vec3,
    core_color: Vec3,
    halo_color: Vec3,
    core_edge: f32,
    halo_edge: f32,
    halo_strength: f32,
) -> Vec3 {
    let alignment = direction.dot(sun_direction);
    let core = smoothstep(core_edge, 1.0, alignment);
    let halo = smoothstep(halo_edge, core_edge, alignment) * halo_strength;
    base + halo_color * halo + core_color * core
}

fn add_moon(
    base: Vec3,
    direction: Vec3,
    celestial_direction: Vec3,
    color: Vec3,
    disk_edge: f32,
    halo_edge: f32,
    halo_strength: f32,
) -> Vec3 {
    let alignment = direction.dot(celestial_direction);
    let disk = smoothstep(disk_edge, 1.0, alignment);
    let halo = smoothstep(halo_edge, disk_edge, alignment) * halo_strength;
    mix(base + color * halo, color, disk)
}

fn atmospheric_gradient(lower: Vec3, horizon: Vec3, zenith: Vec3, height: f32) -> Vec3 {
    let horizon_blend = smoothstep(-0.42, 0.16, height);
    let zenith_blend = smoothstep(0.02, 0.90, height);
    mix(mix(lower, horizon, horizon_blend), zenith, zenith_blend)
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(left: Vec3, right: Vec3, amount: f32) -> Vec3 {
    left * (1.0 - amount) + right * amount
}

fn clamp_color(color: Vec3) -> Vec3 {
    Vec3::new(
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKIES: [SkyType; 5] = [
        SkyType::Sunset,
        SkyType::Cloudy,
        SkyType::StarryNight,
        SkyType::ClearDay,
        SkyType::AuroraNight,
    ];

    #[test]
    fn every_sky_returns_valid_colors() {
        for sky in SKIES {
            for direction in [
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(-0.4, -0.7, 0.2),
            ] {
                let color = sample_sky(direction, sky);
                for channel in [color.x, color.y, color.z] {
                    assert!((0.0..=1.0).contains(&channel));
                }
            }
        }
    }

    #[test]
    fn procedural_skies_are_deterministic_and_vary_by_direction() {
        let first = Vec3::new(0.3, 0.7, -0.2);
        let second = Vec3::new(-0.8, 0.1, 0.4);
        for sky in SKIES {
            assert_eq!(sample_sky(first, sky), sample_sky(first, sky));
            assert_ne!(sample_sky(first, sky), sample_sky(second, sky));
        }
    }

    #[test]
    fn clear_day_has_distinct_horizon_and_zenith() {
        assert_ne!(
            sample_sky(Vec3::new(1.0, 0.0, 0.0), SkyType::ClearDay),
            sample_sky(Vec3::new(0.0, 1.0, 0.0), SkyType::ClearDay)
        );
    }

    #[test]
    fn stars_and_aurora_do_not_depend_on_random_state() {
        let direction = Vec3::new(0.41, 0.52, -0.75);
        assert_eq!(
            sample_sky(direction, SkyType::StarryNight),
            sample_sky(direction, SkyType::StarryNight)
        );
        assert_eq!(
            sample_sky(direction, SkyType::AuroraNight),
            sample_sky(direction, SkyType::AuroraNight)
        );
    }

    #[test]
    fn skies_are_continuous_across_the_horizon() {
        let epsilon = 0.0001;
        for sky in SKIES {
            let above = sample_sky(Vec3::new(0.37, epsilon, -0.93), sky);
            let below = sample_sky(Vec3::new(0.37, -epsilon, -0.93), sky);
            assert!(
                (above - below).length() < 0.12,
                "{sky:?} has a horizon jump"
            );
        }
    }
}
