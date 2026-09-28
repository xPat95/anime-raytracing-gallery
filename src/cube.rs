use crate::{intersect::Intersect, material::Material, ray_intersect::RayIntersect, vec3::Vec3};

const EPSILON: f32 = 0.0001;

pub struct Cube {
    min: Vec3,
    max: Vec3,
    uv_min: Vec3,
    uv_max: Vec3,
    material: Material,
}

impl Cube {
    pub fn from_center_size(center: Vec3, size: f32, material: Material) -> Self {
        let half_size = Vec3::new(size, size, size) * 0.5;

        Self {
            min: center - half_size,
            max: center + half_size,
            uv_min: center - half_size,
            uv_max: center + half_size,
            material,
        }
    }

    pub fn from_block_bounds(
        block_center: Vec3,
        local_min: Vec3,
        local_max: Vec3,
        material: Material,
    ) -> Self {
        let block_min = block_center - Vec3::new(0.5, 0.5, 0.5);

        Self {
            min: block_min + local_min,
            max: block_min + local_max,
            uv_min: block_min,
            uv_max: block_min + Vec3::new(1.0, 1.0, 1.0),
            material,
        }
    }

    fn normal_at(&self, point: Vec3) -> Vec3 {
        if (point.x - self.min.x).abs() < EPSILON {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (point.x - self.max.x).abs() < EPSILON {
            Vec3::new(1.0, 0.0, 0.0)
        } else if (point.y - self.min.y).abs() < EPSILON {
            Vec3::new(0.0, -1.0, 0.0)
        } else if (point.y - self.max.y).abs() < EPSILON {
            Vec3::new(0.0, 1.0, 0.0)
        } else if (point.z - self.min.z).abs() < EPSILON {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }

    fn uv_at(&self, point: Vec3, normal: Vec3) -> (f32, f32) {
        if normal.x > 0.0 {
            (self.uv_max.z - point.z, self.uv_max.y - point.y)
        } else if normal.x < 0.0 {
            (point.z - self.uv_min.z, self.uv_max.y - point.y)
        } else if normal.y > 0.0 {
            (point.x - self.uv_min.x, point.z - self.uv_min.z)
        } else if normal.y < 0.0 {
            (point.x - self.uv_min.x, self.uv_max.z - point.z)
        } else if normal.z > 0.0 {
            (point.x - self.uv_min.x, self.uv_max.y - point.y)
        } else {
            (self.uv_max.x - point.x, self.uv_max.y - point.y)
        }
    }

    #[cfg(test)]
    pub(crate) fn bounds(&self) -> (Vec3, Vec3) {
        (self.min, self.max)
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let mut t_min = f32::NEG_INFINITY;
        let mut t_max = f32::INFINITY;

        for (origin, direction, min, max) in [
            (ray_origin.x, ray_direction.x, self.min.x, self.max.x),
            (ray_origin.y, ray_direction.y, self.min.y, self.max.y),
            (ray_origin.z, ray_direction.z, self.min.z, self.max.z),
        ] {
            if direction.abs() < EPSILON {
                if origin < min || origin > max {
                    return Intersect::empty();
                }

                continue;
            }

            let mut t1 = (min - origin) / direction;
            let mut t2 = (max - origin) / direction;

            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }

            t_min = t_min.max(t1);
            t_max = t_max.min(t2);

            if t_min > t_max {
                return Intersect::empty();
            }
        }

        let distance = if t_min > EPSILON { t_min } else { t_max };

        if distance < EPSILON {
            return Intersect::empty();
        }

        let hit_point = *ray_origin + *ray_direction * distance;
        let normal = self.normal_at(hit_point);
        let (u, v) = self.uv_at(hit_point, normal);
        Intersect::new(distance, normal, u, v, self.material)
    }
}

#[cfg(test)]
mod tests {
    use raylib::prelude::Color;

    use super::*;

    #[test]
    fn opposite_faces_keep_a_consistent_horizontal_orientation() {
        let cube = Cube::from_center_size(Vec3::default(), 2.0, Material::new(Color::WHITE));

        let front = cube.ray_intersect(&Vec3::new(0.5, 0.5, 3.0), &Vec3::new(0.0, 0.0, -1.0));
        let back = cube.ray_intersect(&Vec3::new(0.5, 0.5, -3.0), &Vec3::new(0.0, 0.0, 1.0));

        assert_eq!((front.u, front.v), (1.5, 0.5));
        assert_eq!((back.u, back.v), (0.5, 0.5));
    }

    #[test]
    fn maps_uv_coordinates_on_all_six_faces() {
        let cube = Cube::from_center_size(Vec3::default(), 2.0, Material::new(Color::WHITE));
        let cases = [
            (
                Vec3::new(3.0, 0.5, 0.25),
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                (0.75, 0.5),
            ),
            (
                Vec3::new(-3.0, 0.5, 0.25),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(-1.0, 0.0, 0.0),
                (1.25, 0.5),
            ),
            (
                Vec3::new(0.25, 3.0, 0.5),
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                (1.25, 1.5),
            ),
            (
                Vec3::new(0.25, -3.0, 0.5),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0),
                (1.25, 0.5),
            ),
            (
                Vec3::new(0.25, 0.5, 3.0),
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, 0.0, 1.0),
                (1.25, 0.5),
            ),
            (
                Vec3::new(0.25, 0.5, -3.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 0.0, -1.0),
                (0.75, 0.5),
            ),
        ];

        for (origin, direction, expected_normal, expected_uv) in cases {
            let hit = cube.ray_intersect(&origin, &direction);

            assert_eq!(hit.normal, expected_normal);
            assert_eq!((hit.u, hit.v), expected_uv);
        }
    }

    #[test]
    fn cube_size_controls_texture_repetition() {
        let cube = Cube::from_center_size(Vec3::default(), 2.0, Material::new(Color::WHITE));
        let hit = cube.ray_intersect(&Vec3::new(0.75, 0.75, 3.0), &Vec3::new(0.0, 0.0, -1.0));

        assert_eq!((hit.u, hit.v), (1.75, 0.25));
    }
}
