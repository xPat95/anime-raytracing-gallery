use crate::vec3::Vec3;

pub struct Camera {
    position: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    fov_radians: f32,
}

impl Camera {
    pub fn look_at(position: Vec3, target: Vec3, fov_degrees: f32) -> Self {
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let forward = (target - position).normalize();
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward).normalize();

        Self {
            position,
            forward,
            right,
            up,
            fov_radians: fov_degrees.to_radians(),
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn ray_direction(&self, x: u32, y: u32, width: u32, height: u32) -> Vec3 {
        let aspect_ratio = width as f32 / height as f32;
        let fov_adjustment = (self.fov_radians * 0.5).tan();

        let sensor_x =
            ((((x as f32 + 0.5) / width as f32) * 2.0) - 1.0) * aspect_ratio * fov_adjustment;
        let sensor_y = (1.0 - (((y as f32 + 0.5) / height as f32) * 2.0)) * fov_adjustment;

        (self.forward + self.right * sensor_x + self.up * sensor_y).normalize()
    }
}
