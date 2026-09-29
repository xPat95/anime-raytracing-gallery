use std::cmp::Ordering;

use crate::{cube::Cube, intersect::Intersect, ray_intersect::RayIntersect, vec3::Vec3};

pub const LEAF_SIZE: usize = 4;
const RAY_EPSILON: f32 = 0.0001;
const TRAVERSAL_STACK_SIZE: usize = 64;

#[derive(Clone, Copy, Debug)]
struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    fn empty() -> Self {
        Self {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    fn from_cube(cube: &Cube) -> Self {
        let (min, max) = cube.bounds();
        Self { min, max }
    }

    fn include(&mut self, other: Self) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.min.z = self.min.z.min(other.min.z);
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
        self.max.z = self.max.z.max(other.max.z);
    }

    fn longest_axis(self) -> usize {
        let extent = self.max - self.min;
        if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        }
    }

    fn ray_interval(
        self,
        ray_origin: &Vec3,
        ray_direction: &Vec3,
        max_distance: f32,
    ) -> Option<(f32, f32)> {
        let mut near = f32::NEG_INFINITY;
        let mut far = max_distance;

        for (origin, direction, min, max) in [
            (ray_origin.x, ray_direction.x, self.min.x, self.max.x),
            (ray_origin.y, ray_direction.y, self.min.y, self.max.y),
            (ray_origin.z, ray_direction.z, self.min.z, self.max.z),
        ] {
            if direction.abs() < RAY_EPSILON {
                if origin < min || origin > max {
                    return None;
                }
                continue;
            }

            let mut first = (min - origin) / direction;
            let mut second = (max - origin) / direction;
            if first > second {
                std::mem::swap(&mut first, &mut second);
            }
            near = near.max(first);
            far = far.min(second);
            if near > far {
                return None;
            }
        }

        if far < RAY_EPSILON {
            None
        } else {
            Some((near.max(0.0), far))
        }
    }
}

enum NodeKind {
    Leaf { start: usize, count: usize },
    Branch { left: usize, right: usize },
}

struct BvhNode {
    bounds: Aabb,
    kind: NodeKind,
}

pub struct Bvh {
    nodes: Vec<BvhNode>,
    primitive_indices: Vec<usize>,
    root: Option<usize>,
    max_depth: usize,
}

impl Bvh {
    pub fn build(primitives: &[Cube]) -> Self {
        if primitives.is_empty() {
            return Self {
                nodes: Vec::new(),
                primitive_indices: Vec::new(),
                root: None,
                max_depth: 0,
            };
        }

        let mut bvh = Self {
            nodes: Vec::with_capacity(primitives.len() * 2),
            primitive_indices: Vec::with_capacity(primitives.len()),
            root: None,
            max_depth: 0,
        };
        let mut indices: Vec<usize> = (0..primitives.len()).collect();
        bvh.root = Some(bvh.build_node(&mut indices, primitives, 1));
        bvh
    }

    pub fn closest_hit(
        &self,
        ray_origin: &Vec3,
        ray_direction: &Vec3,
        primitives: &[Cube],
    ) -> Intersect {
        let Some(root) = self.root else {
            return Intersect::empty();
        };
        let mut closest = Intersect::empty();
        let mut closest_index = usize::MAX;
        let Some((root_near, _)) =
            self.nodes[root]
                .bounds
                .ray_interval(ray_origin, ray_direction, closest.distance)
        else {
            return closest;
        };
        let mut node_stack = [0usize; TRAVERSAL_STACK_SIZE];
        let mut near_stack = [0.0f32; TRAVERSAL_STACK_SIZE];
        let mut stack_len = 1;
        node_stack[0] = root;
        near_stack[0] = root_near;

        while stack_len > 0 {
            stack_len -= 1;
            let node_index = node_stack[stack_len];
            let node_near = near_stack[stack_len];
            if node_near > closest.distance {
                continue;
            }
            let node = &self.nodes[node_index];

            match node.kind {
                NodeKind::Leaf { start, count } => {
                    for &primitive_index in &self.primitive_indices[start..start + count] {
                        let hit =
                            primitives[primitive_index].ray_intersect(ray_origin, ray_direction);
                        if hit.is_intersecting
                            && (hit.distance < closest.distance
                                || (hit.distance == closest.distance
                                    && primitive_index < closest_index))
                        {
                            closest = hit;
                            closest_index = primitive_index;
                        }
                    }
                }
                NodeKind::Branch { left, right } => {
                    let left_near = self.nodes[left]
                        .bounds
                        .ray_interval(ray_origin, ray_direction, closest.distance)
                        .map(|interval| interval.0);
                    let right_near = self.nodes[right]
                        .bounds
                        .ray_interval(ray_origin, ray_direction, closest.distance)
                        .map(|interval| interval.0);
                    match (left_near, right_near) {
                        (Some(left_distance), Some(right_distance)) => {
                            if left_distance <= right_distance {
                                push_stack(
                                    &mut node_stack,
                                    &mut near_stack,
                                    &mut stack_len,
                                    right,
                                    right_distance,
                                );
                                push_stack(
                                    &mut node_stack,
                                    &mut near_stack,
                                    &mut stack_len,
                                    left,
                                    left_distance,
                                );
                            } else {
                                push_stack(
                                    &mut node_stack,
                                    &mut near_stack,
                                    &mut stack_len,
                                    left,
                                    left_distance,
                                );
                                push_stack(
                                    &mut node_stack,
                                    &mut near_stack,
                                    &mut stack_len,
                                    right,
                                    right_distance,
                                );
                            }
                        }
                        (Some(distance), None) => push_stack(
                            &mut node_stack,
                            &mut near_stack,
                            &mut stack_len,
                            left,
                            distance,
                        ),
                        (None, Some(distance)) => push_stack(
                            &mut node_stack,
                            &mut near_stack,
                            &mut stack_len,
                            right,
                            distance,
                        ),
                        (None, None) => {}
                    }
                }
            }
        }

        closest
    }

    pub fn is_occluded(
        &self,
        ray_origin: &Vec3,
        ray_direction: &Vec3,
        max_distance: f32,
        primitives: &[Cube],
    ) -> bool {
        let Some(root) = self.root else {
            return false;
        };
        if self.nodes[root]
            .bounds
            .ray_interval(ray_origin, ray_direction, max_distance)
            .is_none()
        {
            return false;
        }
        let mut stack = [0usize; TRAVERSAL_STACK_SIZE];
        let mut stack_len = 1;
        stack[0] = root;

        while stack_len > 0 {
            stack_len -= 1;
            let node_index = stack[stack_len];
            let node = &self.nodes[node_index];

            match node.kind {
                NodeKind::Leaf { start, count } => {
                    for &primitive_index in &self.primitive_indices[start..start + count] {
                        let hit =
                            primitives[primitive_index].ray_intersect(ray_origin, ray_direction);
                        if hit.is_intersecting && hit.distance < max_distance {
                            return true;
                        }
                    }
                }
                NodeKind::Branch { left, right } => {
                    for child in [right, left] {
                        if self.nodes[child]
                            .bounds
                            .ray_interval(ray_origin, ray_direction, max_distance)
                            .is_some()
                        {
                            debug_assert!(stack_len < TRAVERSAL_STACK_SIZE);
                            stack[stack_len] = child;
                            stack_len += 1;
                        }
                    }
                }
            }
        }

        false
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    fn build_node(&mut self, indices: &mut [usize], primitives: &[Cube], depth: usize) -> usize {
        self.max_depth = self.max_depth.max(depth);
        let mut bounds = Aabb::empty();
        let mut centroid_bounds = Aabb::empty();
        for &index in indices.iter() {
            bounds.include(Aabb::from_cube(&primitives[index]));
            let centroid = primitives[index].centroid();
            centroid_bounds.include(Aabb {
                min: centroid,
                max: centroid,
            });
        }

        if indices.len() <= LEAF_SIZE {
            let start = self.primitive_indices.len();
            indices.sort_unstable();
            self.primitive_indices.extend_from_slice(indices);
            let node_index = self.nodes.len();
            self.nodes.push(BvhNode {
                bounds,
                kind: NodeKind::Leaf {
                    start,
                    count: indices.len(),
                },
            });
            return node_index;
        }

        let axis = centroid_bounds.longest_axis();
        indices.sort_unstable_by(|left, right| {
            axis_value(primitives[*left].centroid(), axis)
                .partial_cmp(&axis_value(primitives[*right].centroid(), axis))
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.cmp(right))
        });
        let middle = indices.len() / 2;
        let (left_indices, right_indices) = indices.split_at_mut(middle);
        let left = self.build_node(left_indices, primitives, depth + 1);
        let right = self.build_node(right_indices, primitives, depth + 1);
        let node_index = self.nodes.len();
        self.nodes.push(BvhNode {
            bounds,
            kind: NodeKind::Branch { left, right },
        });
        node_index
    }
}

#[inline]
fn push_stack(
    nodes: &mut [usize; TRAVERSAL_STACK_SIZE],
    distances: &mut [f32; TRAVERSAL_STACK_SIZE],
    len: &mut usize,
    node: usize,
    distance: f32,
) {
    debug_assert!(*len < TRAVERSAL_STACK_SIZE);
    nodes[*len] = node;
    distances[*len] = distance;
    *len += 1;
}

fn axis_value(value: Vec3, axis: usize) -> f32 {
    match axis {
        0 => value.x,
        1 => value.y,
        _ => value.z,
    }
}

#[cfg(test)]
mod tests {
    use raylib::prelude::Color;

    use super::*;
    use crate::{block_geometry, material::Material};

    fn material() -> Material {
        Material::new(Color::WHITE)
    }

    fn linear_hit(origin: &Vec3, direction: &Vec3, primitives: &[Cube]) -> Intersect {
        let mut closest = Intersect::empty();
        for primitive in primitives {
            let hit = primitive.ray_intersect(origin, direction);
            if hit.is_intersecting && hit.distance < closest.distance {
                closest = hit;
            }
        }
        closest
    }

    #[test]
    fn empty_bvh_returns_no_hit_or_occlusion() {
        let bvh = Bvh::build(&[]);

        assert!(
            !bvh.closest_hit(&Vec3::default(), &Vec3::new(0.0, 0.0, 1.0), &[])
                .is_intersecting
        );
        assert!(!bvh.is_occluded(&Vec3::default(), &Vec3::new(0.0, 0.0, 1.0), 10.0, &[]));
    }

    #[test]
    fn root_miss_returns_no_hit() {
        let primitives = [Cube::from_center_size(Vec3::default(), 1.0, material())];
        let bvh = Bvh::build(&primitives);

        assert!(
            !bvh.closest_hit(
                &Vec3::new(3.0, 3.0, 3.0),
                &Vec3::new(1.0, 0.0, 0.0),
                &primitives
            )
            .is_intersecting
        );
    }

    #[test]
    fn bvh_matches_linear_closest_hit() {
        let primitives = [
            Cube::from_center_size(Vec3::new(0.0, 0.0, 6.0), 1.0, material()),
            Cube::from_center_size(Vec3::new(0.0, 0.0, 3.0), 1.0, material()),
        ];
        let bvh = Bvh::build(&primitives);
        let origin = Vec3::default();
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let linear = linear_hit(&origin, &direction, &primitives);
        let accelerated = bvh.closest_hit(&origin, &direction, &primitives);

        assert_eq!(accelerated.distance, linear.distance);
        assert_eq!(accelerated.normal, linear.normal);
    }

    #[test]
    fn bvh_matches_linear_for_deterministic_ray_set() {
        let primitives = [
            Cube::from_center_size(Vec3::new(-2.0, 1.0, 4.0), 1.0, material()),
            Cube::from_center_size(Vec3::new(0.0, -1.0, 6.0), 2.0, material()),
            Cube::from_center_size(Vec3::new(3.0, 2.0, 9.0), 1.5, material()),
            Cube::from_center_size(Vec3::new(1.0, 0.0, -3.0), 0.5, material()),
        ];
        let bvh = Bvh::build(&primitives);
        let rays = [
            (Vec3::default(), Vec3::new(-2.0, 1.0, 4.0).normalize()),
            (Vec3::default(), Vec3::new(0.0, -1.0, 6.0).normalize()),
            (
                Vec3::new(5.0, 2.0, 0.0),
                Vec3::new(-2.0, 0.0, 9.0).normalize(),
            ),
            (Vec3::default(), Vec3::new(0.0, 1.0, 0.0)),
            (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
        ];

        for (origin, direction) in rays {
            let linear = linear_hit(&origin, &direction, &primitives);
            let accelerated = bvh.closest_hit(&origin, &direction, &primitives);
            assert_eq!(accelerated.is_intersecting, linear.is_intersecting);
            assert_eq!(accelerated.distance, linear.distance);
            assert_eq!(accelerated.normal, linear.normal);
        }
    }

    #[test]
    fn shadow_any_hit_respects_max_distance() {
        let primitives = [Cube::from_center_size(
            Vec3::new(0.0, 0.0, 8.0),
            1.0,
            material(),
        )];
        let bvh = Bvh::build(&primitives);
        let origin = Vec3::default();
        let direction = Vec3::new(0.0, 0.0, 1.0);

        assert!(!bvh.is_occluded(&origin, &direction, 5.0, &primitives));
        assert!(bvh.is_occluded(&origin, &direction, 10.0, &primitives));
    }

    #[test]
    fn bvh_intersects_special_aabb_parts() {
        let primitives = block_geometry::slab(
            Vec3::new(0.0, 0.0, 3.0),
            "minecraft:blackstone_slab[type=bottom,waterlogged=false]",
            material(),
        )
        .unwrap();
        let bvh = Bvh::build(&primitives);
        let hit = bvh.closest_hit(&Vec3::default(), &Vec3::new(0.0, 0.0, 1.0), &primitives);

        assert!(hit.is_intersecting);
        assert_eq!(hit.distance, 2.5);
    }
}
