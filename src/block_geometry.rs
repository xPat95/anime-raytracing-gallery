use crate::{cube::Cube, material::Material, vec3::Vec3};

const HALF: f32 = 0.5;
const WALL_MIN: f32 = 5.0 / 16.0;
const WALL_MAX: f32 = 11.0 / 16.0;
const WALL_LOW_HEIGHT: f32 = 14.0 / 16.0;

pub fn slab(center: Vec3, state: &str, material: Material) -> Result<Vec<Cube>, String> {
    let bounds = match required_property(state, "type")? {
        "bottom" => (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, HALF, 1.0)),
        "top" => (Vec3::new(0.0, HALF, 0.0), Vec3::new(1.0, 1.0, 1.0)),
        "double" => (Vec3::default(), Vec3::new(1.0, 1.0, 1.0)),
        value => return Err(format!("unsupported slab type '{value}' in '{state}'")),
    };

    Ok(vec![Cube::from_block_bounds(
        center, bounds.0, bounds.1, material,
    )])
}

pub fn stairs(center: Vec3, state: &str, material: Material) -> Result<Vec<Cube>, String> {
    let shape = required_property(state, "shape")?;
    if shape != "straight" {
        return Err(format!("unsupported stair shape '{shape}' in '{state}'"));
    }

    let (base_min_y, base_max_y, step_min_y, step_max_y) = match required_property(state, "half")? {
        "bottom" => (0.0, HALF, HALF, 1.0),
        "top" => (HALF, 1.0, 0.0, HALF),
        value => return Err(format!("unsupported stair half '{value}' in '{state}'")),
    };
    let (step_min, step_max) = match required_property(state, "facing")? {
        "north" => (
            Vec3::new(0.0, step_min_y, 0.0),
            Vec3::new(1.0, step_max_y, HALF),
        ),
        "south" => (
            Vec3::new(0.0, step_min_y, HALF),
            Vec3::new(1.0, step_max_y, 1.0),
        ),
        "west" => (
            Vec3::new(0.0, step_min_y, 0.0),
            Vec3::new(HALF, step_max_y, 1.0),
        ),
        "east" => (
            Vec3::new(HALF, step_min_y, 0.0),
            Vec3::new(1.0, step_max_y, 1.0),
        ),
        value => return Err(format!("unsupported stair facing '{value}' in '{state}'")),
    };

    Ok(vec![
        Cube::from_block_bounds(
            center,
            Vec3::new(0.0, base_min_y, 0.0),
            Vec3::new(1.0, base_max_y, 1.0),
            material,
        ),
        Cube::from_block_bounds(center, step_min, step_max, material),
    ])
}

pub fn wall(center: Vec3, state: &str, material: Material) -> Result<Vec<Cube>, String> {
    let mut parts = Vec::new();

    match required_property(state, "up")? {
        "true" => parts.push(Cube::from_block_bounds(
            center,
            Vec3::new(0.25, 0.0, 0.25),
            Vec3::new(0.75, 1.0, 0.75),
            material,
        )),
        "false" => {}
        value => return Err(format!("unsupported wall up value '{value}' in '{state}'")),
    }

    add_wall_connection(&mut parts, center, state, "north", material)?;
    add_wall_connection(&mut parts, center, state, "south", material)?;
    add_wall_connection(&mut parts, center, state, "west", material)?;
    add_wall_connection(&mut parts, center, state, "east", material)?;

    Ok(parts)
}

fn add_wall_connection(
    parts: &mut Vec<Cube>,
    center: Vec3,
    state: &str,
    direction: &str,
    material: Material,
) -> Result<(), String> {
    let height = match required_property(state, direction)? {
        "none" => return Ok(()),
        "low" => WALL_LOW_HEIGHT,
        "tall" => 1.0,
        value => {
            return Err(format!(
                "unsupported wall connection '{direction}={value}' in '{state}'"
            ));
        }
    };
    let (min, max) = match direction {
        "north" => (
            Vec3::new(WALL_MIN, 0.0, 0.0),
            Vec3::new(WALL_MAX, height, HALF),
        ),
        "south" => (
            Vec3::new(WALL_MIN, 0.0, HALF),
            Vec3::new(WALL_MAX, height, 1.0),
        ),
        "west" => (
            Vec3::new(0.0, 0.0, WALL_MIN),
            Vec3::new(HALF, height, WALL_MAX),
        ),
        "east" => (
            Vec3::new(HALF, 0.0, WALL_MIN),
            Vec3::new(1.0, height, WALL_MAX),
        ),
        _ => unreachable!(),
    };

    parts.push(Cube::from_block_bounds(center, min, max, material));
    Ok(())
}

fn required_property<'a>(state: &'a str, name: &str) -> Result<&'a str, String> {
    let properties = state
        .split_once('[')
        .and_then(|value| value.1.strip_suffix(']'))
        .ok_or_else(|| format!("block state has no properties: '{state}'"))?;

    properties
        .split(',')
        .filter_map(|property| property.split_once('='))
        .find_map(|(key, value)| (key == name).then_some(value))
        .ok_or_else(|| format!("block state has no '{name}' property: '{state}'"))
}

#[cfg(test)]
mod tests {
    use raylib::prelude::Color;

    use super::*;

    fn material() -> Material {
        Material::new(Color::WHITE)
    }

    #[test]
    fn bottom_and_top_slabs_use_opposite_halves() {
        let bottom = slab(
            Vec3::default(),
            "minecraft:blackstone_slab[type=bottom,waterlogged=false]",
            material(),
        )
        .unwrap();
        let top = slab(
            Vec3::default(),
            "minecraft:blackstone_slab[type=top,waterlogged=false]",
            material(),
        )
        .unwrap();

        assert_eq!(bottom[0].bounds().1.y, 0.0);
        assert_eq!(top[0].bounds().0.y, 0.0);
    }

    #[test]
    fn stair_facing_and_half_select_the_step_box() {
        let east_bottom = stairs(
            Vec3::default(),
            "minecraft:blackstone_stairs[facing=east,half=bottom,shape=straight,waterlogged=false]",
            material(),
        )
        .unwrap();
        let north_top = stairs(
            Vec3::default(),
            "minecraft:blackstone_stairs[facing=north,half=top,shape=straight,waterlogged=false]",
            material(),
        )
        .unwrap();

        assert_eq!(east_bottom[1].bounds().0, Vec3::new(0.0, 0.0, -0.5));
        assert_eq!(north_top[1].bounds().1, Vec3::new(0.5, 0.0, 0.0));
    }

    #[test]
    fn wall_only_adds_present_connections() {
        let parts = wall(
            Vec3::default(),
            "minecraft:blackstone_wall[east=low,north=none,south=none,up=true,waterlogged=false,west=none]",
            material(),
        )
        .unwrap();

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].bounds().0, Vec3::new(0.0, -0.5, -0.1875));
        assert_eq!(parts[1].bounds().1, Vec3::new(0.5, 0.375, 0.1875));
    }
}
