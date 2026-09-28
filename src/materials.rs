use crate::{material::Material, texture::Texture};

const STONE_TEXTURE: usize = 0;
const MOSS_BLOCK_TEXTURE: usize = 1;
const OBSIDIAN_TEXTURE: usize = 2;
const SMOOTH_QUARTZ_TEXTURE: usize = 3;
const NETHER_PORTAL_TEXTURE: usize = 4;
const BLACKSTONE_TEXTURE: usize = 5;

pub struct MaterialCatalog {
    pub stone: Material,
    pub moss_block: Material,
    pub smooth_quartz: Material,
    pub blackstone: Material,
    pub obsidian: Material,
    pub nether_portal: Material,
}

impl MaterialCatalog {
    pub fn load() -> Result<(Self, Vec<Texture>), String> {
        let textures = vec![
            Texture::load("assets/textures/stone.png")?,
            Texture::load("assets/textures/moss_block.png")?,
            Texture::load("assets/textures/obsidian.png")?,
            Texture::load("assets/textures/smooth_quartz.png")?,
            Texture::load_first_frame("assets/textures/nether_portal.png", 16)?,
            Texture::load("assets/textures/blackstone.png")?,
        ];
        Ok((Self::configured(), textures))
    }

    fn configured() -> Self {
        Self {
            stone: Material::textured(STONE_TEXTURE, 0.75, 0.10, 0.00, 0.03),
            moss_block: Material::textured(MOSS_BLOCK_TEXTURE, 0.80, 0.05, 0.00, 0.01),
            smooth_quartz: Material::textured(SMOOTH_QUARTZ_TEXTURE, 0.90, 0.30, 0.00, 0.08),
            blackstone: Material::textured(BLACKSTONE_TEXTURE, 0.65, 0.12, 0.00, 0.04),
            obsidian: Material::textured(OBSIDIAN_TEXTURE, 0.60, 0.65, 0.00, 0.30),
            nether_portal: Material::textured(NETHER_PORTAL_TEXTURE, 0.85, 0.60, 0.35, 0.15),
        }
    }

    pub fn for_block(&self, block_name: &str) -> Result<Option<Material>, String> {
        let material = match block_name {
            "minecraft:air" => return Ok(None),
            "minecraft:stone" => self.stone,
            "minecraft:moss_block" => self.moss_block,
            "minecraft:obsidian" => self.obsidian,
            "minecraft:smooth_quartz" => self.smooth_quartz,
            "minecraft:nether_portal" => self.nether_portal,
            "minecraft:blackstone_slab"
            | "minecraft:blackstone_stairs"
            | "minecraft:blackstone_wall" => self.blackstone,
            _ => return Err(format!("scene contains unsupported block '{block_name}'")),
        };

        Ok(Some(material))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_properties_are_normalized() {
        let catalog = MaterialCatalog::configured();
        for material in [
            catalog.stone,
            catalog.moss_block,
            catalog.smooth_quartz,
            catalog.blackstone,
            catalog.obsidian,
            catalog.nether_portal,
        ] {
            for value in [
                material.albedo,
                material.specular,
                material.transparency,
                material.reflectivity,
            ] {
                assert!((0.0..=1.0).contains(&value));
            }
        }
    }

    #[test]
    fn configured_materials_store_expected_optical_values() {
        let catalog = MaterialCatalog::configured();

        assert_eq!(catalog.stone.albedo, 0.75);
        assert_eq!(catalog.nether_portal.transparency, 0.35);
        assert_eq!(catalog.stone.transparency, 0.0);
        assert_eq!(catalog.moss_block.transparency, 0.0);
        assert!(catalog.obsidian.reflectivity > catalog.stone.reflectivity);
        assert!(catalog.moss_block.specular < catalog.smooth_quartz.specular);
    }
}
