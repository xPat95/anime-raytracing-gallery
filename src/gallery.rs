use raylib::prelude::Color;

use crate::{
    camera::CameraConfig,
    light::{LightingConfig, PointLight},
    materials::{BlockGeometry, MaterialConfig, TextureConfig},
    vec3::Vec3,
};

pub struct SceneConfig {
    pub id: &'static str,
    pub display_name: &'static str,
    pub scene_path: &'static str,
    pub textures: Vec<TextureConfig>,
    pub materials: Vec<MaterialConfig>,
    pub camera: CameraConfig,
    pub lighting: LightingConfig,
}

pub fn available_scenes() -> Vec<SceneConfig> {
    vec![black_clover_skull()]
}

fn black_clover_skull() -> SceneConfig {
    let textures = vec![
        texture("stone", "assets/textures/stone.png"),
        texture("moss_block", "assets/textures/moss_block.png"),
        texture("obsidian", "assets/textures/obsidian.png"),
        texture("smooth_quartz", "assets/textures/smooth_quartz.png"),
        TextureConfig {
            id: "nether_portal",
            path: "assets/textures/nether_portal.png",
            first_frame_height: Some(16),
        },
        texture("blackstone", "assets/textures/blackstone.png"),
    ];
    let materials = vec![
        material(
            "minecraft:stone",
            "stone",
            BlockGeometry::OpaqueCube,
            0.75,
            0.10,
            0.00,
            0.03,
        ),
        material(
            "minecraft:moss_block",
            "moss_block",
            BlockGeometry::OpaqueCube,
            0.80,
            0.05,
            0.00,
            0.01,
        ),
        material(
            "minecraft:obsidian",
            "obsidian",
            BlockGeometry::OpaqueCube,
            0.60,
            0.65,
            0.00,
            0.30,
        ),
        material(
            "minecraft:smooth_quartz",
            "smooth_quartz",
            BlockGeometry::OpaqueCube,
            0.90,
            0.30,
            0.00,
            0.08,
        ),
        material(
            "minecraft:nether_portal",
            "nether_portal",
            BlockGeometry::Portal,
            0.85,
            0.60,
            0.35,
            0.15,
        ),
        material(
            "minecraft:blackstone_slab",
            "blackstone",
            BlockGeometry::Slab,
            0.65,
            0.12,
            0.00,
            0.04,
        ),
        material(
            "minecraft:blackstone_stairs",
            "blackstone",
            BlockGeometry::Stairs,
            0.65,
            0.12,
            0.00,
            0.04,
        ),
        material(
            "minecraft:blackstone_wall",
            "blackstone",
            BlockGeometry::Wall,
            0.65,
            0.12,
            0.00,
            0.04,
        ),
    ];

    SceneConfig {
        id: "black_clover_skull",
        display_name: "Black Clover - Skull",
        scene_path: "assets/scenes/proyecto.scene",
        textures,
        materials,
        camera: CameraConfig::from_position(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(145.0, 95.0, 180.0),
            5.0,
            420.0,
        ),
        lighting: LightingConfig {
            light: PointLight {
                position: Vec3::new(-90.0, 125.0, 140.0),
                color: Color::new(255, 244, 224, 255),
                intensity: 1.15,
            },
            ambient_intensity: 0.10,
            shadow_bias: 0.001,
            phong_shininess: 32.0,
        },
    }
}

fn texture(id: &'static str, path: &'static str) -> TextureConfig {
    TextureConfig {
        id,
        path,
        first_frame_height: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn material(
    block_name: &'static str,
    texture_id: &'static str,
    geometry: BlockGeometry,
    albedo: f32,
    specular: f32,
    transparency: f32,
    reflectivity: f32,
) -> MaterialConfig {
    MaterialConfig {
        block_name,
        texture_id,
        geometry,
        albedo,
        specular,
        transparency,
        reflectivity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bvh::Bvh, materials::MaterialCatalog, scene};

    #[test]
    fn black_clover_is_the_only_registered_scene() {
        let scenes = available_scenes();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].id, "black_clover_skull");
        assert_eq!(scenes[0].display_name, "Black Clover - Skull");
        assert_eq!(scenes[0].scene_path, "assets/scenes/proyecto.scene");
    }

    #[test]
    fn black_clover_keeps_its_camera_and_materials() {
        let scene = black_clover_skull();
        assert_eq!(scene.camera.target, Vec3::new(0.0, 0.0, 0.0));
        assert!((scene.camera.radius - 249.89998).abs() < 0.001);
        assert!((scene.camera.yaw.to_degrees() - 38.853).abs() < 0.001);
        assert!((scene.camera.pitch.to_degrees() - 22.343).abs() < 0.001);
        assert_eq!(scene.camera.min_radius, 5.0);
        assert_eq!(scene.camera.max_radius, 420.0);
        assert_eq!(scene.materials.len(), 8);
        assert_eq!(scene.textures.len(), 6);
        assert_eq!(
            scene.lighting.light.position,
            Vec3::new(-90.0, 125.0, 140.0)
        );
        assert_eq!(scene.lighting.ambient_intensity, 0.10);
    }

    #[test]
    fn registered_scene_loads_materials_geometry_and_bvh() {
        let config = black_clover_skull();
        let (materials, textures) =
            MaterialCatalog::load(&config.textures, &config.materials).unwrap();
        let scene = scene::load(
            config.scene_path,
            |block_name| materials.for_block(block_name),
            true,
        )
        .unwrap();
        let bvh = Bvh::build(&scene.primitives);

        assert_eq!(textures.len(), 6);
        assert_eq!(scene.block_count(), 23_432);
        assert_eq!(scene.slabs, 45);
        assert_eq!(scene.stairs, 58);
        assert_eq!(scene.walls, 64);
        assert_eq!(scene.primitives.len(), 23_443);
        assert!(bvh.node_count() > 0);
    }
}
