use raylib::prelude::Color;

use crate::{
    camera::CameraConfig,
    ground::{GroundConfig, GroundKind},
    light::{LightingConfig, PointLight},
    materials::{BlockGeometry, MaterialConfig, TextureConfig},
    sky::SkyType,
    vec3::Vec3,
};

pub struct SceneConfig {
    pub id: &'static str,
    pub display_name: &'static str,
    pub scene_path: &'static str,
    pub rotate_y_180: bool,
    pub textures: Vec<TextureConfig>,
    pub materials: Vec<MaterialConfig>,
    pub camera: CameraConfig,
    pub lighting: LightingConfig,
    pub sky: SkyType,
    pub ground: GroundConfig,
}

pub fn available_scenes() -> Vec<SceneConfig> {
    vec![
        black_clover_skull(),
        shenlong(),
        kurama(),
        pochita(),
        lapras(),
    ]
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
        stone_material(),
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
            0.45,
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
        rotate_y_180: false,
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
        sky: SkyType::Sunset,
        ground: solid_ground(-42.501, Color::new(43, 48, 52, 255), 0.72, 0.04, 0.01),
    }
}

fn shenlong() -> SceneConfig {
    let textures = vec![
        texture("green_concrete", "assets/textures/green_concrete.png"),
        texture("terracotta", "assets/textures/terracotta.png"),
        texture(
            "orange_stained_glass",
            "assets/textures/orange_stained_glass.png",
        ),
        texture("glowstone", "assets/textures/glowstone.png"),
        texture("gray_terracotta", "assets/textures/gray_terracotta.png"),
        texture("red_stained_glass", "assets/textures/red_stained_glass.png"),
    ];
    let materials = vec![
        material(
            "minecraft:green_concrete",
            "green_concrete",
            BlockGeometry::OpaqueCube,
            0.78,
            0.08,
            0.00,
            0.02,
        ),
        material(
            "minecraft:terracotta",
            "terracotta",
            BlockGeometry::OpaqueCube,
            0.75,
            0.10,
            0.00,
            0.03,
        ),
        refractive_material(
            "minecraft:orange_stained_glass",
            "orange_stained_glass",
            BlockGeometry::Cube,
            0.88,
            0.45,
            0.35,
            0.12,
            1.5,
        ),
        glowstone_material(),
        material(
            "minecraft:gray_terracotta",
            "gray_terracotta",
            BlockGeometry::OpaqueCube,
            0.72,
            0.10,
            0.00,
            0.03,
        ),
        red_stained_glass_material(),
    ];

    SceneConfig {
        id: "shenlong",
        display_name: "Shenlong",
        scene_path: "assets/scenes/shenlong.scene",
        rotate_y_180: false,
        textures,
        materials,
        camera: CameraConfig::from_position(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(205.0, 140.0, 248.0),
            5.0,
            550.0,
        ),
        lighting: LightingConfig {
            light: PointLight {
                position: Vec3::new(140.0, 220.0, 200.0),
                color: Color::new(255, 244, 224, 255),
                intensity: 1.15,
            },
            ambient_intensity: 0.14,
            shadow_bias: 0.001,
            phong_shininess: 32.0,
        },
        sky: SkyType::Cloudy,
        ground: solid_ground(-75.001, Color::new(74, 118, 62, 255), 0.78, 0.04, 0.01),
    }
}

fn kurama() -> SceneConfig {
    let textures = vec![
        texture("orange_concrete", "assets/textures/orange_concrete.png"),
        texture("black_concrete", "assets/textures/black_concrete.png"),
        texture("white_concrete", "assets/textures/white_concrete.png"),
        texture("pink_concrete", "assets/textures/pink_concrete.png"),
        texture("glowstone", "assets/textures/glowstone.png"),
        texture("red_stained_glass", "assets/textures/red_stained_glass.png"),
    ];
    let materials = vec![
        orange_concrete_material(),
        black_concrete_material(),
        white_concrete_material(),
        pink_concrete_material(),
        glowstone_material(),
        red_stained_glass_material(),
    ];

    SceneConfig {
        id: "kurama",
        display_name: "Kurama",
        scene_path: "assets/scenes/kurama.scene",
        rotate_y_180: false,
        textures,
        materials,
        camera: CameraConfig::from_position(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(211.0, 134.0, 246.0),
            12.0,
            850.0,
        ),
        lighting: LightingConfig {
            light: PointLight {
                position: Vec3::new(260.0, 320.0, 300.0),
                color: Color::new(255, 244, 224, 255),
                intensity: 1.20,
            },
            ambient_intensity: 0.14,
            shadow_bias: 0.001,
            phong_shininess: 32.0,
        },
        sky: SkyType::StarryNight,
        ground: solid_ground(-85.001, Color::new(82, 84, 86, 255), 0.72, 0.06, 0.02),
    }
}

fn pochita() -> SceneConfig {
    let textures = vec![
        texture("orange_concrete", "assets/textures/orange_concrete.png"),
        texture("white_concrete", "assets/textures/white_concrete.png"),
        texture("stone", "assets/textures/stone.png"),
        texture("black_concrete", "assets/textures/black_concrete.png"),
        texture("red_wool", "assets/textures/red_wool.png"),
    ];
    let materials = vec![
        orange_concrete_material(),
        white_concrete_material(),
        stone_material(),
        black_concrete_material(),
        red_wool_material(),
    ];

    SceneConfig {
        id: "pochita",
        display_name: "Pochita",
        scene_path: "assets/scenes/pochita.scene",
        rotate_y_180: true,
        textures,
        materials,
        camera: CameraConfig::from_position(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(78.0, 53.0, 111.0),
            5.0,
            320.0,
        ),
        lighting: LightingConfig {
            light: PointLight {
                position: Vec3::new(90.0, 130.0, 120.0),
                color: Color::new(255, 244, 224, 255),
                intensity: 1.15,
            },
            ambient_intensity: 0.14,
            shadow_bias: 0.001,
            phong_shininess: 32.0,
        },
        sky: SkyType::ClearDay,
        ground: solid_ground(-32.501, Color::new(92, 170, 70, 255), 0.88, 0.05, 0.015),
    }
}

fn lapras() -> SceneConfig {
    let textures = vec![
        texture("light_blue_wool", "assets/textures/light_blue_wool.png"),
        texture("ochre_froglight", "assets/textures/ochre_froglight.png"),
        texture("diorite", "assets/textures/diorite.png"),
        texture("pink_concrete", "assets/textures/pink_concrete.png"),
        texture("brown_concrete", "assets/textures/brown_concrete.png"),
        texture("red_wool", "assets/textures/red_wool.png"),
    ];
    let materials = vec![
        material(
            "minecraft:light_blue_wool",
            "light_blue_wool",
            BlockGeometry::OpaqueCube,
            0.78,
            0.03,
            0.00,
            0.01,
        ),
        material(
            "minecraft:ochre_froglight",
            "ochre_froglight",
            BlockGeometry::OpaqueCube,
            0.92,
            0.22,
            0.00,
            0.07,
        ),
        material(
            "minecraft:diorite",
            "diorite",
            BlockGeometry::OpaqueCube,
            0.80,
            0.15,
            0.00,
            0.04,
        ),
        pink_concrete_material(),
        material(
            "minecraft:brown_concrete",
            "brown_concrete",
            BlockGeometry::OpaqueCube,
            0.75,
            0.08,
            0.00,
            0.02,
        ),
        red_wool_material(),
    ];

    SceneConfig {
        id: "lapras",
        display_name: "Lapras",
        scene_path: "assets/scenes/lapras.scene",
        rotate_y_180: false,
        textures,
        materials,
        camera: CameraConfig::from_position(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(95.0, 62.0, 111.0),
            5.0,
            450.0,
        ),
        lighting: LightingConfig {
            light: PointLight {
                position: Vec3::new(130.0, 170.0, 150.0),
                color: Color::new(255, 244, 224, 255),
                intensity: 1.15,
            },
            ambient_intensity: 0.14,
            shadow_bias: 0.001,
            phong_shininess: 32.0,
        },
        sky: SkyType::AuroraNight,
        ground: GroundConfig {
            kind: GroundKind::Water,
            height: -30.0,
            tint: Color::new(190, 232, 240, 255),
            albedo: 0.18,
            specular: 0.55,
            transparency: 0.72,
            reflectivity: 0.28,
            ior: Some(1.33),
        },
    }
}

fn stone_material() -> MaterialConfig {
    material(
        "minecraft:stone",
        "stone",
        BlockGeometry::OpaqueCube,
        0.75,
        0.10,
        0.00,
        0.03,
    )
}

fn orange_concrete_material() -> MaterialConfig {
    material(
        "minecraft:orange_concrete",
        "orange_concrete",
        BlockGeometry::OpaqueCube,
        0.80,
        0.08,
        0.00,
        0.02,
    )
}

fn black_concrete_material() -> MaterialConfig {
    material(
        "minecraft:black_concrete",
        "black_concrete",
        BlockGeometry::OpaqueCube,
        0.72,
        0.12,
        0.00,
        0.04,
    )
}

fn white_concrete_material() -> MaterialConfig {
    material(
        "minecraft:white_concrete",
        "white_concrete",
        BlockGeometry::OpaqueCube,
        0.90,
        0.25,
        0.00,
        0.06,
    )
}

fn pink_concrete_material() -> MaterialConfig {
    material(
        "minecraft:pink_concrete",
        "pink_concrete",
        BlockGeometry::OpaqueCube,
        0.82,
        0.12,
        0.00,
        0.03,
    )
}

fn red_wool_material() -> MaterialConfig {
    material(
        "minecraft:red_wool",
        "red_wool",
        BlockGeometry::OpaqueCube,
        0.78,
        0.03,
        0.00,
        0.01,
    )
}

fn glowstone_material() -> MaterialConfig {
    material(
        "minecraft:glowstone",
        "glowstone",
        BlockGeometry::OpaqueCube,
        0.95,
        0.25,
        0.00,
        0.08,
    )
}

fn red_stained_glass_material() -> MaterialConfig {
    refractive_material(
        "minecraft:red_stained_glass",
        "red_stained_glass",
        BlockGeometry::Cube,
        0.88,
        0.45,
        0.35,
        0.12,
        1.5,
    )
}

fn texture(id: &'static str, path: &'static str) -> TextureConfig {
    TextureConfig {
        id,
        path,
        first_frame_height: None,
    }
}

fn solid_ground(
    height: f32,
    tint: Color,
    albedo: f32,
    specular: f32,
    reflectivity: f32,
) -> GroundConfig {
    GroundConfig {
        kind: GroundKind::Solid,
        height,
        tint,
        albedo,
        specular,
        transparency: 0.0,
        reflectivity,
        ior: None,
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
        ior: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn refractive_material(
    block_name: &'static str,
    texture_id: &'static str,
    geometry: BlockGeometry,
    albedo: f32,
    specular: f32,
    transparency: f32,
    reflectivity: f32,
    ior: f32,
) -> MaterialConfig {
    MaterialConfig {
        ior: Some(ior),
        ..material(
            block_name,
            texture_id,
            geometry,
            albedo,
            specular,
            transparency,
            reflectivity,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bvh::Bvh, materials::MaterialCatalog, scene};

    #[test]
    fn real_scenes_are_registered() {
        let scenes = available_scenes();
        assert_eq!(scenes.len(), 5);
        assert_eq!(scenes[0].id, "black_clover_skull");
        assert_eq!(scenes[0].display_name, "Black Clover - Skull");
        assert_eq!(scenes[0].scene_path, "assets/scenes/proyecto.scene");
        assert_eq!(scenes[1].id, "shenlong");
        assert_eq!(scenes[1].display_name, "Shenlong");
        assert_eq!(scenes[1].scene_path, "assets/scenes/shenlong.scene");
        assert_eq!(scenes[2].id, "kurama");
        assert_eq!(scenes[2].display_name, "Kurama");
        assert_eq!(scenes[2].scene_path, "assets/scenes/kurama.scene");
        assert_eq!(scenes[3].id, "pochita");
        assert_eq!(scenes[3].display_name, "Pochita");
        assert_eq!(scenes[3].scene_path, "assets/scenes/pochita.scene");
        assert!(scenes[3].rotate_y_180);
        assert_eq!(scenes[4].id, "lapras");
        assert_eq!(scenes[4].display_name, "Lapras");
        assert_eq!(scenes[4].scene_path, "assets/scenes/lapras.scene");
        assert!(
            scenes
                .iter()
                .enumerate()
                .all(|(index, scene)| scene.rotate_y_180 == (index == 3))
        );
        assert_eq!(scenes[0].sky, SkyType::Sunset);
        assert_eq!(scenes[1].sky, SkyType::Cloudy);
        assert_eq!(scenes[2].sky, SkyType::StarryNight);
        assert_eq!(scenes[3].sky, SkyType::ClearDay);
        assert_eq!(scenes[4].sky, SkyType::AuroraNight);
        assert_eq!(scenes[0].ground.kind, GroundKind::Solid);
        assert_eq!(scenes[0].ground.height, -42.501);
        assert_eq!(scenes[1].ground.kind, GroundKind::Solid);
        assert_eq!(scenes[1].ground.height, -75.001);
        assert_eq!(scenes[2].ground.height, -85.001);
        assert_eq!(scenes[3].ground.height, -32.501);
        assert!(scenes[0].ground.tint.r < scenes[2].ground.tint.r);
        assert!(scenes[0].ground.tint.g < scenes[2].ground.tint.g);
        assert!(scenes[1].ground.tint.g > scenes[1].ground.tint.r);
        assert!(scenes[3].ground.tint.g > scenes[1].ground.tint.g);
        assert!(scenes[..4].iter().all(|scene| {
            scene
                .textures
                .iter()
                .all(|texture| texture.id != "deepslate" && texture.id != "grass_block_top")
        }));
        assert_eq!(scenes[4].ground.kind, GroundKind::Water);
        assert_eq!(scenes[4].ground.height, -30.0);
        assert!(scenes[4].ground.transparency > 0.0);
        assert!(scenes[4].ground.reflectivity > 0.0);
        assert_eq!(scenes[4].ground.ior, Some(1.33));
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
        let portal = scene
            .materials
            .iter()
            .find(|material| material.block_name == "minecraft:nether_portal")
            .unwrap();
        assert_eq!(portal.transparency, 0.45);
        assert_eq!(portal.ior, None);
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

    #[test]
    fn shenlong_loads_its_six_materials_and_builds_a_bvh() {
        let config = shenlong();
        let mut camera = crate::camera::Camera::new(config.camera);
        let initial_position = camera.position();
        let (materials, textures) =
            MaterialCatalog::load(&config.textures, &config.materials).unwrap();
        for block_name in [
            "minecraft:green_concrete",
            "minecraft:terracotta",
            "minecraft:orange_stained_glass",
            "minecraft:glowstone",
            "minecraft:gray_terracotta",
            "minecraft:red_stained_glass",
        ] {
            assert!(materials.for_block(block_name).unwrap().is_some());
        }
        let scene = scene::load(
            config.scene_path,
            |block_name| materials.for_block(block_name),
            true,
        )
        .unwrap();
        let bvh = Bvh::build(&scene.primitives);

        assert_eq!(textures.len(), 6);
        assert_eq!(scene.block_count(), 30_915);
        assert_eq!(scene.primitives.len(), 29_283);
        assert!(bvh.node_count() > 0);
        assert!(camera.radius() > config.camera.min_radius);
        assert!(camera.radius() < config.camera.max_radius);

        camera.orbit(20.0, -10.0);
        camera.pan(15.0, -5.0);
        camera.zoom(3.0);
        assert!(camera.reset());
        assert!((camera.position() - initial_position).length() < 0.001);
        assert!((camera.radius() - config.camera.radius).abs() < 0.001);

        let orange_glass = config
            .materials
            .iter()
            .find(|material| material.block_name == "minecraft:orange_stained_glass")
            .unwrap();
        let glowstone = config
            .materials
            .iter()
            .find(|material| material.block_name == "minecraft:glowstone")
            .unwrap();
        assert_eq!(orange_glass.transparency, 0.35);
        assert_eq!(orange_glass.ior, Some(1.5));
        assert_eq!(orange_glass.geometry, BlockGeometry::Cube);
        assert_eq!(glowstone.albedo, 0.95);
        let red_glass = config
            .materials
            .iter()
            .find(|material| material.block_name == "minecraft:red_stained_glass")
            .unwrap();
        assert_eq!(red_glass.transparency, 0.35);
        assert_eq!(red_glass.ior, Some(1.5));
    }

    #[test]
    fn kurama_loads_its_six_materials_and_builds_a_bvh() {
        let config = kurama();
        let mut camera = crate::camera::Camera::new(config.camera);
        let initial_position = camera.position();
        let (materials, textures) =
            MaterialCatalog::load(&config.textures, &config.materials).unwrap();
        for block_name in [
            "minecraft:orange_concrete",
            "minecraft:black_concrete",
            "minecraft:white_concrete",
            "minecraft:pink_concrete",
            "minecraft:glowstone",
            "minecraft:red_stained_glass",
        ] {
            assert!(materials.for_block(block_name).unwrap().is_some());
        }
        let scene = scene::load(
            config.scene_path,
            |block_name| materials.for_block(block_name),
            true,
        )
        .unwrap();
        let bvh = Bvh::build(&scene.primitives);

        assert_eq!(textures.len(), 6);
        assert_eq!(scene.block_count(), 81_380);
        assert!(!scene.primitives.is_empty());
        assert!(bvh.node_count() > 0);
        assert!(camera.radius() > config.camera.min_radius);
        assert!(camera.radius() < config.camera.max_radius);

        camera.orbit(20.0, -10.0);
        camera.pan(15.0, -5.0);
        camera.zoom(3.0);
        assert!(camera.reset());
        assert!((camera.position() - initial_position).length() < 0.001);
        assert!((camera.radius() - config.camera.radius).abs() < 0.001);
    }

    #[test]
    fn pochita_loads_its_five_materials_and_builds_a_bvh() {
        let config = pochita();
        let mut camera = crate::camera::Camera::new(config.camera);
        let initial_position = camera.position();
        let (materials, textures) =
            MaterialCatalog::load(&config.textures, &config.materials).unwrap();
        for block_name in [
            "minecraft:orange_concrete",
            "minecraft:white_concrete",
            "minecraft:stone",
            "minecraft:black_concrete",
            "minecraft:red_wool",
        ] {
            assert!(materials.for_block(block_name).unwrap().is_some());
        }
        let scene = scene::load(
            config.scene_path,
            |block_name| materials.for_block(block_name),
            true,
        )
        .unwrap();
        let bvh = Bvh::build(&scene.primitives);

        assert_eq!(textures.len(), 5);
        assert_eq!(scene.block_count(), 12_833);
        assert!(!scene.primitives.is_empty());
        assert!(bvh.node_count() > 0);
        assert!(camera.radius() > config.camera.min_radius);
        assert!(camera.radius() < config.camera.max_radius);

        camera.orbit(20.0, -10.0);
        camera.pan(15.0, -5.0);
        camera.zoom(3.0);
        assert!(camera.reset());
        assert!((camera.position() - initial_position).length() < 0.001);
        assert!((camera.radius() - config.camera.radius).abs() < 0.001);

        let red_wool = config
            .materials
            .iter()
            .find(|material| material.block_name == "minecraft:red_wool")
            .unwrap();
        assert_eq!(red_wool.specular, 0.03);
        assert_eq!(red_wool.transparency, 0.0);
        assert_eq!(red_wool.reflectivity, 0.01);
    }

    #[test]
    fn lapras_loads_its_six_materials_and_builds_a_bvh() {
        let config = lapras();
        let mut camera = crate::camera::Camera::new(config.camera);
        let initial_position = camera.position();
        let (materials, textures) =
            MaterialCatalog::load(&config.textures, &config.materials).unwrap();
        for block_name in [
            "minecraft:light_blue_wool",
            "minecraft:ochre_froglight",
            "minecraft:diorite",
            "minecraft:pink_concrete",
            "minecraft:brown_concrete",
            "minecraft:red_wool",
        ] {
            assert!(materials.for_block(block_name).unwrap().is_some());
        }
        let scene = scene::load(
            config.scene_path,
            |block_name| materials.for_block(block_name),
            true,
        )
        .unwrap();
        let bvh = Bvh::build(&scene.primitives);

        assert_eq!(textures.len(), 6);
        assert_eq!(scene.block_count(), 14_635);
        assert!(!scene.primitives.is_empty());
        assert!(bvh.node_count() > 0);
        assert!(camera.radius() > config.camera.min_radius);
        assert!(camera.radius() < config.camera.max_radius);

        camera.orbit(20.0, -10.0);
        camera.pan(15.0, -5.0);
        camera.zoom(3.0);
        assert!(camera.reset());
        assert!((camera.position() - initial_position).length() < 0.001);
        assert!((camera.radius() - config.camera.radius).abs() < 0.001);
    }
}
