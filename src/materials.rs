use std::collections::HashMap;

use crate::{material::Material, texture::Texture};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockGeometry {
    OpaqueCube,
    Portal,
    Slab,
    Stairs,
    Wall,
}

#[derive(Clone, Copy)]
pub struct TextureConfig {
    pub id: &'static str,
    pub path: &'static str,
    pub first_frame_height: Option<u32>,
}

#[derive(Clone, Copy)]
pub struct MaterialConfig {
    pub block_name: &'static str,
    pub texture_id: &'static str,
    pub geometry: BlockGeometry,
    pub albedo: f32,
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
}

#[derive(Clone, Copy)]
pub struct BlockDefinition {
    pub material: Material,
    pub geometry: BlockGeometry,
}

pub struct MaterialCatalog {
    definitions: HashMap<&'static str, BlockDefinition>,
}

impl MaterialCatalog {
    pub fn load(
        texture_configs: &[TextureConfig],
        material_configs: &[MaterialConfig],
    ) -> Result<(Self, Vec<Texture>), String> {
        let mut texture_indices = HashMap::new();
        let mut textures = Vec::with_capacity(texture_configs.len());

        for config in texture_configs {
            if texture_indices.contains_key(config.id) {
                return Err(format!("duplicate texture id '{}'", config.id));
            }
            let texture = match config.first_frame_height {
                Some(height) => Texture::load_first_frame(config.path, height)?,
                None => Texture::load(config.path)?,
            };
            texture_indices.insert(config.id, textures.len());
            textures.push(texture);
        }

        let mut definitions = HashMap::new();
        for config in material_configs {
            let texture_index = *texture_indices.get(config.texture_id).ok_or_else(|| {
                format!(
                    "material '{}' references unknown texture '{}'",
                    config.block_name, config.texture_id
                )
            })?;
            let definition = BlockDefinition {
                material: Material::textured(
                    texture_index,
                    config.albedo,
                    config.specular,
                    config.transparency,
                    config.reflectivity,
                ),
                geometry: config.geometry,
            };
            if definitions.insert(config.block_name, definition).is_some() {
                return Err(format!(
                    "duplicate material for block '{}'",
                    config.block_name
                ));
            }
        }

        Ok((Self { definitions }, textures))
    }

    pub fn for_block(&self, block_name: &str) -> Result<Option<BlockDefinition>, String> {
        if block_name == "minecraft:air" {
            return Ok(None);
        }
        self.definitions
            .get(block_name)
            .copied()
            .map(Some)
            .ok_or_else(|| format!("scene contains unsupported block '{block_name}'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_requires_a_configured_texture() {
        let result = MaterialCatalog::load(
            &[],
            &[MaterialConfig {
                block_name: "minecraft:test",
                texture_id: "missing",
                geometry: BlockGeometry::OpaqueCube,
                albedo: 1.0,
                specular: 0.0,
                transparency: 0.0,
                reflectivity: 0.0,
            }],
        );

        assert!(result.is_err());
    }
}
