use crate::{intersect::Intersect, material::Material, ray_intersect::RayIntersect, vec3::Vec3};

const EPSILON: f32 = 0.0001;

pub struct Cube {
    min: Vec3,
    max: Vec3,
    material: Material,
}

impl Cube {
    pub fn from_center_size(center: Vec3, size: f32, material: Material) -> Self {
        let half_size = Vec3::new(size, size, size) * 0.5;

        Self {
            min: center - half_size,
            max: center + half_size,
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
        Intersect::new(distance, self.normal_at(hit_point), self.material)
    }
}
