use std::f32::consts::TAU;

use crate::vec3::Vec3;

const INITIAL_POSITION: Vec3 = Vec3 {
    x: 145.0,
    y: 95.0,
    z: 180.0,
};
const TARGET: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};
const FOV_DEGREES: f32 = 43.0;
const MIN_PITCH: f32 = -80.0_f32.to_radians();
const MAX_PITCH: f32 = 80.0_f32.to_radians();
const MIN_RADIUS: f32 = 150.0;
const MAX_RADIUS: f32 = 420.0;
pub const MOUSE_SENSITIVITY: f32 = 0.005;
pub const ZOOM_SENSITIVITY: f32 = 12.0;

pub struct Camera {
    target: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    position: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    fov_radians: f32,
}

impl Camera {
    pub fn scene() -> Self {
        let offset = INITIAL_POSITION - TARGET;
        let radius = offset.length();
        let yaw = offset.x.atan2(offset.z);
        let pitch = (offset.y / radius).asin();
        let mut camera = Self {
            target: TARGET,
            radius,
            yaw,
            pitch,
            position: INITIAL_POSITION,
            forward: Vec3::default(),
            right: Vec3::default(),
            up: Vec3::default(),
            fov_radians: FOV_DEGREES.to_radians(),
        };
        camera.rebuild_basis();
        camera
    }

    pub fn orbit(&mut self, mouse_delta_x: f32, mouse_delta_y: f32) -> bool {
        if mouse_delta_x == 0.0 && mouse_delta_y == 0.0 {
            return false;
        }

        self.yaw = (self.yaw + mouse_delta_x * MOUSE_SENSITIVITY).rem_euclid(TAU);
        self.pitch = (self.pitch - mouse_delta_y * MOUSE_SENSITIVITY).clamp(MIN_PITCH, MAX_PITCH);
        self.rebuild_basis();
        true
    }

    pub fn zoom(&mut self, wheel_movement: f32) -> bool {
        if wheel_movement == 0.0 {
            return false;
        }

        let previous_radius = self.radius;
        self.radius =
            (self.radius - wheel_movement * ZOOM_SENSITIVITY).clamp(MIN_RADIUS, MAX_RADIUS);
        self.rebuild_basis();
        self.radius != previous_radius
    }

    pub fn reset(&mut self) -> bool {
        let initial = Self::scene();
        let changed = self.position != initial.position;
        *self = initial;
        changed
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn yaw_degrees(&self) -> f32 {
        self.yaw.to_degrees()
    }

    pub fn pitch_degrees(&self) -> f32 {
        self.pitch.to_degrees()
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn ray_direction(&self, x: u32, y: u32, width: u32, height: u32) -> Vec3 {
        let aspect_ratio = width as f32 / height as f32;
        let fov_adjustment = (self.fov_radians * 0.5).tan();

        let sensor_x =
            ((((x as f32 + 0.5) / width as f32) * 2.0) - 1.0) * aspect_ratio * fov_adjustment;
        let sensor_y = (1.0 - (((y as f32 + 0.5) / height as f32) * 2.0)) * fov_adjustment;

        (self.forward + self.right * sensor_x + self.up * sensor_y).normalize()
    }

    fn rebuild_basis(&mut self) {
        let cos_pitch = self.pitch.cos();
        let orbit_direction = Vec3::new(
            cos_pitch * self.yaw.sin(),
            self.pitch.sin(),
            cos_pitch * self.yaw.cos(),
        );
        self.position = self.target + orbit_direction * self.radius;
        self.forward = (self.target - self.position).normalize();
        self.right = self.forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        self.up = self.right.cross(self.forward).normalize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approximately_equal(left: f32, right: f32) -> bool {
        (left - right).abs() < 0.001
    }

    #[test]
    fn initial_orbit_reconstructs_previous_camera_position() {
        let camera = Camera::scene();

        assert!(approximately_equal(camera.position.x, INITIAL_POSITION.x));
        assert!(approximately_equal(camera.position.y, INITIAL_POSITION.y));
        assert!(approximately_equal(camera.position.z, INITIAL_POSITION.z));
    }

    #[test]
    fn pitch_and_zoom_stay_within_limits() {
        let mut camera = Camera::scene();

        camera.orbit(0.0, -100_000.0);
        assert!(approximately_equal(camera.pitch, MAX_PITCH));
        camera.orbit(0.0, 100_000.0);
        assert!(approximately_equal(camera.pitch, MIN_PITCH));

        camera.zoom(100_000.0);
        assert_eq!(camera.radius, MIN_RADIUS);
        camera.zoom(-100_000.0);
        assert_eq!(camera.radius, MAX_RADIUS);
    }

    #[test]
    fn reset_restores_initial_orbit() {
        let mut camera = Camera::scene();
        camera.orbit(50.0, -25.0);
        camera.zoom(2.0);

        assert!(camera.reset());
        assert!(approximately_equal(camera.position.x, INITIAL_POSITION.x));
        assert!(approximately_equal(camera.position.y, INITIAL_POSITION.y));
        assert!(approximately_equal(camera.position.z, INITIAL_POSITION.z));
    }

    #[test]
    fn camera_basis_remains_orthonormal_after_orbit() {
        let mut camera = Camera::scene();
        camera.orbit(300.0, -100.0);

        for vector in [camera.forward, camera.right, camera.up] {
            assert!(approximately_equal(vector.length(), 1.0));
        }
        assert!(approximately_equal(camera.forward.dot(camera.right), 0.0));
        assert!(approximately_equal(camera.forward.dot(camera.up), 0.0));
        assert!(approximately_equal(camera.right.dot(camera.up), 0.0));
    }
}
