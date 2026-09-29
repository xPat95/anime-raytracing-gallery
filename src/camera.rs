use std::f32::consts::TAU;

use crate::vec3::Vec3;

#[cfg(test)]
const INITIAL_POSITION: Vec3 = Vec3 {
    x: 145.0,
    y: 95.0,
    z: 180.0,
};
#[cfg(test)]
const TARGET: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};
const FOV_DEGREES: f32 = 43.0;
const MIN_PITCH: f32 = -80.0_f32.to_radians();
const MAX_PITCH: f32 = 80.0_f32.to_radians();
pub const MOUSE_SENSITIVITY: f32 = 0.005;
pub const KEYBOARD_ORBIT_SPEED: f32 = 60.0_f32.to_radians();
pub const MOUSE_PAN_SENSITIVITY: f32 = 0.1;
pub const KEYBOARD_PAN_SPEED: f32 = 30.0;
pub const ZOOM_SENSITIVITY: f32 = 12.0;

#[derive(Clone, Copy)]
pub struct CameraConfig {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
    pub min_radius: f32,
    pub max_radius: f32,
}

impl CameraConfig {
    #[cfg(test)]
    pub fn from_position(target: Vec3, position: Vec3, min_radius: f32, max_radius: f32) -> Self {
        let offset = position - target;
        let radius = offset.length();
        Self {
            target,
            yaw: offset.x.atan2(offset.z),
            pitch: (offset.y / radius).asin(),
            radius,
            min_radius,
            max_radius,
        }
    }
}

pub struct Camera {
    initial_config: CameraConfig,
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
    pub fn new(config: CameraConfig) -> Self {
        let mut camera = Self {
            initial_config: config,
            target: config.target,
            radius: config.radius,
            yaw: config.yaw,
            pitch: config.pitch,
            position: Vec3::default(),
            forward: Vec3::default(),
            right: Vec3::default(),
            up: Vec3::default(),
            fov_radians: FOV_DEGREES.to_radians(),
        };
        camera.rebuild_basis();
        camera
    }

    #[cfg(test)]
    pub fn scene() -> Self {
        Self::new(CameraConfig::from_position(
            TARGET,
            INITIAL_POSITION,
            5.0,
            420.0,
        ))
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

    pub fn orbit_keyboard(&mut self, horizontal: f32, vertical: f32, frame_time: f32) -> bool {
        if horizontal == 0.0 && vertical == 0.0 {
            return false;
        }

        self.yaw = (self.yaw + horizontal * KEYBOARD_ORBIT_SPEED * frame_time).rem_euclid(TAU);
        self.pitch =
            (self.pitch + vertical * KEYBOARD_ORBIT_SPEED * frame_time).clamp(MIN_PITCH, MAX_PITCH);
        self.rebuild_basis();
        true
    }

    pub fn pan(&mut self, mouse_delta_x: f32, mouse_delta_y: f32) -> bool {
        if mouse_delta_x == 0.0 && mouse_delta_y == 0.0 {
            return false;
        }

        self.translate_target(
            mouse_delta_x * MOUSE_PAN_SENSITIVITY,
            -mouse_delta_y * MOUSE_PAN_SENSITIVITY,
        );
        true
    }

    pub fn pan_keyboard(&mut self, horizontal: f32, vertical: f32, frame_time: f32) -> bool {
        if horizontal == 0.0 && vertical == 0.0 {
            return false;
        }

        self.translate_target(
            horizontal * KEYBOARD_PAN_SPEED * frame_time,
            vertical * KEYBOARD_PAN_SPEED * frame_time,
        );
        true
    }

    pub fn zoom(&mut self, wheel_movement: f32) -> bool {
        if wheel_movement == 0.0 {
            return false;
        }

        let previous_radius = self.radius;
        self.radius = (self.radius - wheel_movement * ZOOM_SENSITIVITY).clamp(
            self.initial_config.min_radius,
            self.initial_config.max_radius,
        );
        self.rebuild_basis();
        self.radius != previous_radius
    }

    pub fn reset(&mut self) -> bool {
        let config = self.initial_config;
        let changed = self.target != config.target
            || self.radius != config.radius
            || self.yaw != config.yaw
            || self.pitch != config.pitch;
        *self = Self::new(config);
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

    fn translate_target(&mut self, horizontal: f32, vertical: f32) {
        self.target = self.target + self.right * horizontal + self.up * vertical;
        self.rebuild_basis();
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
        assert_eq!(camera.radius, camera.initial_config.min_radius);
        camera.zoom(-100_000.0);
        assert_eq!(camera.radius, camera.initial_config.max_radius);
    }

    #[test]
    fn reset_restores_initial_orbit() {
        let mut camera = Camera::scene();
        camera.orbit(50.0, -25.0);
        camera.zoom(2.0);
        camera.pan(30.0, -20.0);

        assert!(camera.reset());
        assert_eq!(camera.target, TARGET);
        assert!(approximately_equal(camera.radius, Camera::scene().radius));
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

    #[test]
    fn keyboard_orbit_uses_yaw_pitch_and_pitch_limits() {
        let mut camera = Camera::scene();
        let initial_yaw = camera.yaw;
        let initial_pitch = camera.pitch;

        camera.orbit_keyboard(1.0, 1.0, 0.5);

        assert!(camera.yaw > initial_yaw);
        assert!(camera.pitch > initial_pitch);
        camera.orbit_keyboard(0.0, 1.0, 100.0);
        assert!(approximately_equal(camera.pitch, MAX_PITCH));
    }

    #[test]
    fn pan_moves_target_without_changing_orbit_parameters() {
        let mut camera = Camera::scene();
        let initial_target = camera.target;
        let initial_position = camera.position;
        let initial_yaw = camera.yaw;
        let initial_pitch = camera.pitch;
        let initial_radius = camera.radius;

        camera.pan(20.0, -10.0);

        let translation = camera.target - initial_target;
        assert_ne!(camera.target, initial_target);
        assert!(((camera.position - initial_position) - translation).length() < 0.001);
        assert_eq!(camera.yaw, initial_yaw);
        assert_eq!(camera.pitch, initial_pitch);
        assert_eq!(camera.radius, initial_radius);
    }

    #[test]
    fn orbit_after_pan_uses_displaced_target() {
        let mut camera = Camera::scene();
        camera.pan_keyboard(1.0, 1.0, 0.5);
        let displaced_target = camera.target;

        camera.orbit_keyboard(1.0, 0.0, 0.5);

        assert_eq!(camera.target, displaced_target);
        assert!(approximately_equal(
            (camera.position - camera.target).length(),
            camera.radius
        ));
    }
}
