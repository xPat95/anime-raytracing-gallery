use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{block_geometry, cube::Cube, material::Material, vec3::Vec3};

pub struct Scene {
    pub primitives: Vec<Cube>,
    pub dimensions: [u32; 3],
    pub offset: Vec3,
    pub full_blocks: usize,
    pub slabs: usize,
    pub stairs: usize,
    pub walls: usize,
}

impl Scene {
    pub fn block_count(&self) -> usize {
        self.full_blocks + self.slabs + self.stairs + self.walls
    }
}

pub fn load(
    path: impl AsRef<Path>,
    material_for_block: impl Fn(&str) -> Result<Option<Material>, String>,
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
    let mut scene = Scene {
        primitives: Vec::new(),
        dimensions,
        offset,
        full_blocks: 0,
        slabs: 0,
        stairs: 0,
        walls: 0,
    };

    for (position, block_state) in blocks {
        let block_name = block_state
            .split_once('[')
            .map_or(block_state.as_str(), |value| value.0);
        let center = Vec3::new(
            position[0] as f32 + offset.x,
            position[1] as f32 + offset.y,
            position[2] as f32 + offset.z,
        );

        match block_name {
            "minecraft:blackstone_slab" => {
                let material = required_material(&material_for_block, block_name)?;
                scene
                    .primitives
                    .extend(block_geometry::slab(center, &block_state, material)?);
                scene.slabs += 1;
            }
            "minecraft:blackstone_stairs" => {
                let material = required_material(&material_for_block, block_name)?;
                scene
                    .primitives
                    .extend(block_geometry::stairs(center, &block_state, material)?);
                scene.stairs += 1;
            }
            "minecraft:blackstone_wall" => {
                let material = required_material(&material_for_block, block_name)?;
                scene
                    .primitives
                    .extend(block_geometry::wall(center, &block_state, material)?);
                scene.walls += 1;
            }
            _ => {
                if let Some(material) = material_for_block(block_name)? {
                    scene
                        .primitives
                        .push(Cube::from_center_size(center, 1.0, material));
                    scene.full_blocks += 1;
                }
            }
        }
    }

    Ok(scene)
}

fn required_material(
    material_for_block: &impl Fn(&str) -> Result<Option<Material>, String>,
    block_name: &str,
) -> Result<Material, String> {
    material_for_block(block_name)?.ok_or_else(|| {
        format!("special block '{block_name}' requires a material but none was provided")
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
