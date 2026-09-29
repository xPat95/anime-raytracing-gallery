use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{
    block_geometry,
    cube::Cube,
    materials::{BlockDefinition, BlockGeometry},
    vec3::Vec3,
};

pub struct Scene {
    pub primitives: Vec<Cube>,
    pub dimensions: [u32; 3],
    pub offset: Vec3,
    pub full_blocks: usize,
    pub retained_full_blocks: usize,
    pub enclosed_full_blocks: usize,
    pub culled_full_blocks: usize,
    pub portal_blocks: usize,
    pub slabs: usize,
    pub stairs: usize,
    pub walls: usize,
}

impl Scene {
    pub fn block_count(&self) -> usize {
        self.full_blocks + self.portal_blocks + self.slabs + self.stairs + self.walls
    }

    pub fn primitives_before_culling(&self) -> usize {
        self.primitives.len() + self.culled_full_blocks
    }

    pub fn rotate_y_180(&mut self) {
        for primitive in &mut self.primitives {
            primitive.rotate_y_180();
        }
        self.offset.x = -self.offset.x;
        self.offset.z = -self.offset.z;
    }
}

pub fn load(
    path: impl AsRef<Path>,
    definition_for_block: impl Fn(&str) -> Result<Option<BlockDefinition>, String>,
    cull_fully_enclosed: bool,
) -> Result<Scene, String> {
    let path = path.as_ref();
    let file = File::open(path)
        .map_err(|error| format!("could not open scene '{}': {error}", path.display()))?;
    let mut dimensions = None;
    let mut offset = None;
    let mut blocks = Vec::new();

    for (line_index, line) in BufReader::new(file).lines().enumerate() {
        let line_number = line_index + 1;
        let line = line.map_err(|error| {
            format!(
                "could not read scene '{}' at line {line_number}: {error}",
                path.display()
            )
        })?;

        if let Some(value) = line.strip_prefix("# dimensions=") {
            dimensions = Some(parse_triplet(value, path, line_number)?);
        } else if let Some(value) = line.strip_prefix("# center_offset=") {
            let values = parse_float_triplet(value, path, line_number)?;
            offset = Some(Vec3::new(values[0], values[1], values[2]));
        } else if !line.is_empty() && !line.starts_with('#') {
            blocks.push(parse_block(&line, path, line_number)?);
        }
    }

    let dimensions = dimensions.ok_or_else(|| {
        format!(
            "scene '{}' does not contain a dimensions header",
            path.display()
        )
    })?;
    let offset = offset.ok_or_else(|| {
        format!(
            "scene '{}' does not contain a center_offset header",
            path.display()
        )
    })?;
    let mut resolved_blocks = Vec::with_capacity(blocks.len());
    for (position, block_state) in blocks {
        if let Some(definition) = definition_for_block(block_name(&block_state))? {
            resolved_blocks.push((position, block_state, definition));
        }
    }
    let opaque_full_positions: HashSet<[i32; 3]> = resolved_blocks
        .iter()
        .filter_map(|(position, _, definition)| {
            is_opaque_full_block(*definition).then_some(*position)
        })
        .collect();
    let mut scene = Scene {
        primitives: Vec::new(),
        dimensions,
        offset,
        full_blocks: 0,
        retained_full_blocks: 0,
        enclosed_full_blocks: 0,
        culled_full_blocks: 0,
        portal_blocks: 0,
        slabs: 0,
        stairs: 0,
        walls: 0,
    };

    for (position, block_state, definition) in resolved_blocks {
        let center = Vec3::new(
            position[0] as f32 + offset.x,
            position[1] as f32 + offset.y,
            position[2] as f32 + offset.z,
        );

        match definition.geometry {
            BlockGeometry::Cube => {
                scene.full_blocks += 1;
                scene
                    .primitives
                    .push(Cube::from_center_size(center, 1.0, definition.material));
                scene.retained_full_blocks += 1;
            }
            BlockGeometry::Slab => {
                scene.primitives.extend(block_geometry::slab(
                    center,
                    &block_state,
                    definition.material,
                )?);
                scene.slabs += 1;
            }
            BlockGeometry::Stairs => {
                scene.primitives.extend(block_geometry::stairs(
                    center,
                    &block_state,
                    definition.material,
                )?);
                scene.stairs += 1;
            }
            BlockGeometry::Wall => {
                scene.primitives.extend(block_geometry::wall(
                    center,
                    &block_state,
                    definition.material,
                )?);
                scene.walls += 1;
            }
            BlockGeometry::Portal => {
                scene
                    .primitives
                    .push(Cube::from_center_size(center, 1.0, definition.material));
                scene.portal_blocks += 1;
            }
            BlockGeometry::OpaqueCube => {
                scene.full_blocks += 1;
                let fully_enclosed = is_fully_enclosed(position, &opaque_full_positions);
                if fully_enclosed {
                    scene.enclosed_full_blocks += 1;
                }
                if cull_fully_enclosed && fully_enclosed {
                    scene.culled_full_blocks += 1;
                } else {
                    scene
                        .primitives
                        .push(Cube::from_center_size(center, 1.0, definition.material));
                    scene.retained_full_blocks += 1;
                }
            }
        }
    }

    Ok(scene)
}

fn block_name(block_state: &str) -> &str {
    block_state
        .split_once('[')
        .map_or(block_state, |value| value.0)
}

fn is_opaque_full_block(definition: BlockDefinition) -> bool {
    definition.geometry == BlockGeometry::OpaqueCube && definition.material.is_opaque()
}

fn is_fully_enclosed(position: [i32; 3], opaque_full_positions: &HashSet<[i32; 3]>) -> bool {
    const NEIGHBOR_OFFSETS: [[i32; 3]; 6] = [
        [1, 0, 0],
        [-1, 0, 0],
        [0, 1, 0],
        [0, -1, 0],
        [0, 0, 1],
        [0, 0, -1],
    ];

    NEIGHBOR_OFFSETS.iter().all(|offset| {
        opaque_full_positions.contains(&[
            position[0] + offset[0],
            position[1] + offset[1],
            position[2] + offset[2],
        ])
    })
}

fn parse_block(line: &str, path: &Path, line_number: usize) -> Result<([i32; 3], String), String> {
    let mut fields = line.splitn(4, '\t');
    let x = parse_value(fields.next(), "x", path, line_number)?;
    let y = parse_value(fields.next(), "y", path, line_number)?;
    let z = parse_value(fields.next(), "z", path, line_number)?;
    let block_state = fields.next().ok_or_else(|| {
        format!(
            "scene '{}' has no block state at line {line_number}",
            path.display()
        )
    })?;

    Ok(([x, y, z], block_state.to_owned()))
}

fn parse_triplet(value: &str, path: &Path, line_number: usize) -> Result<[u32; 3], String> {
    let mut fields = value.split(',');
    Ok([
        parse_value(fields.next(), "width", path, line_number)?,
        parse_value(fields.next(), "height", path, line_number)?,
        parse_value(fields.next(), "length", path, line_number)?,
    ])
}

fn parse_float_triplet(value: &str, path: &Path, line_number: usize) -> Result<[f32; 3], String> {
    let mut fields = value.split(',');
    Ok([
        parse_value(fields.next(), "offset x", path, line_number)?,
        parse_value(fields.next(), "offset y", path, line_number)?,
        parse_value(fields.next(), "offset z", path, line_number)?,
    ])
}

fn parse_value<T: std::str::FromStr>(
    value: Option<&str>,
    name: &str,
    path: &Path,
    line_number: usize,
) -> Result<T, String> {
    value.and_then(|value| value.parse().ok()).ok_or_else(|| {
        format!(
            "scene '{}' has an invalid {name} at line {line_number}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use raylib::prelude::Color;

    use super::*;

    fn definition(geometry: BlockGeometry, transparency: f32) -> BlockDefinition {
        let mut material = crate::material::Material::new(Color::WHITE);
        material.transparency = transparency;
        BlockDefinition { material, geometry }
    }

    fn enclosed_positions() -> HashSet<[i32; 3]> {
        [
            [0, 0, 0],
            [1, 0, 0],
            [-1, 0, 0],
            [0, 1, 0],
            [0, -1, 0],
            [0, 0, 1],
            [0, 0, -1],
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn cube_with_six_opaque_full_neighbors_is_enclosed() {
        assert!(is_fully_enclosed([0, 0, 0], &enclosed_positions()));
    }

    #[test]
    fn missing_neighbor_keeps_cube() {
        let mut positions = enclosed_positions();
        positions.remove(&[0, 1, 0]);

        assert!(!is_fully_enclosed([0, 0, 0], &positions));
    }

    #[test]
    fn surface_cube_is_not_enclosed() {
        assert!(!is_fully_enclosed([1, 0, 0], &enclosed_positions()));
    }

    #[test]
    fn partial_blocks_and_portal_are_not_opaque_full_neighbors() {
        for geometry in [
            BlockGeometry::Cube,
            BlockGeometry::Slab,
            BlockGeometry::Stairs,
            BlockGeometry::Wall,
            BlockGeometry::Portal,
        ] {
            assert!(!is_opaque_full_block(definition(geometry, 0.0)));

            let mut positions = enclosed_positions();
            positions.remove(&[1, 0, 0]);
            if is_opaque_full_block(definition(geometry, 0.0)) {
                positions.insert([1, 0, 0]);
            }
            assert!(!is_fully_enclosed([0, 0, 0], &positions));
        }
    }

    #[test]
    fn opaque_full_cube_can_occlude_an_adjacent_face() {
        assert!(is_opaque_full_block(definition(
            BlockGeometry::OpaqueCube,
            0.0
        )));
    }

    #[test]
    fn transparent_neighbor_does_not_fully_enclose_a_cube() {
        let mut positions = enclosed_positions();
        positions.remove(&[1, 0, 0]);
        let transparent = definition(BlockGeometry::OpaqueCube, 0.35);
        if is_opaque_full_block(transparent) {
            positions.insert([1, 0, 0]);
        }

        assert!(!is_fully_enclosed([0, 0, 0], &positions));
    }

    #[test]
    fn transparent_full_cubes_are_conservative_occlusion_boundaries() {
        let opaque = definition(BlockGeometry::OpaqueCube, 0.0);
        let transparent = definition(BlockGeometry::Cube, 0.35);

        assert!(is_opaque_full_block(opaque));
        assert!(!is_opaque_full_block(transparent));
        assert!(!is_opaque_full_block(definition(
            BlockGeometry::OpaqueCube,
            0.35
        )));
    }
}
