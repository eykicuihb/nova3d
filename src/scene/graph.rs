use super::Transform;
use crate::math::Mat4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Node {
    local: Transform,
    parent: Option<NodeId>,
}

#[derive(Debug, Default)]
pub struct SceneGraph {
    nodes: Vec<Node>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, parent: Option<NodeId>, local: Transform) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node { local, parent });
        id
    }

    pub fn set_local_transform(&mut self, id: NodeId, local: Transform) {
        self.nodes[id.0].local = local;
    }

    pub fn local_transform(&self, id: NodeId) -> &Transform {
        &self.nodes[id.0].local
    }

    pub fn world_transform(&self, id: NodeId) -> Mat4 {
        let mut world = Mat4::identity();
        let mut current = Some(id);

        while let Some(current_id) = current {
            let node = &self.nodes[current_id.0];
            world = node.local.to_mat4().multiply(world);
            current = node.parent;
        }

        world
    }
}

#[cfg(test)]
mod tests {
    use super::SceneGraph;
    use crate::math::{Mat4, Quat, Vec3};
    use crate::scene::Transform;

    const EPSILON: f32 = 1.0e-6;

    fn assert_mat4_near(actual: Mat4, expected: Mat4) {
        for (actual, expected) in actual.data.iter().zip(expected.data.iter()) {
            assert!(
                (actual - expected).abs() < EPSILON,
                "{actual} != {expected}"
            );
        }
    }

    #[test]
    fn root_world_transform_equals_local_transform() {
        let mut graph = SceneGraph::new();
        let local = Transform {
            translation: Vec3::new(2.0, 3.0, 4.0),
            ..Transform::IDENTITY
        };

        let root = graph.add_node(None, local);

        assert_mat4_near(graph.world_transform(root), local.to_mat4());
    }

    #[test]
    fn child_world_transform_composes_parent_translation() {
        let mut graph = SceneGraph::new();
        let parent = graph.add_node(
            None,
            Transform {
                translation: Vec3::new(2.0, 3.0, 4.0),
                ..Transform::IDENTITY
            },
        );
        let child = graph.add_node(
            Some(parent),
            Transform {
                translation: Vec3::new(5.0, 6.0, 7.0),
                ..Transform::IDENTITY
            },
        );

        assert_mat4_near(
            graph.world_transform(child),
            Mat4::from_translation(Vec3::new(2.0, 3.0, 4.0))
                .multiply(Mat4::from_translation(Vec3::new(5.0, 6.0, 7.0))),
        );
    }

    #[test]
    fn two_level_hierarchy_composes_rotation_then_translation() {
        let mut graph = SceneGraph::new();
        let parent = graph.add_node(
            None,
            Transform {
                rotation: Quat::from_axis_angle(
                    Vec3::new(0.0, 0.0, 1.0),
                    std::f32::consts::FRAC_PI_2,
                ),
                ..Transform::IDENTITY
            },
        );
        let child_local = Transform {
            translation: Vec3::new(1.0, 0.0, 0.0),
            ..Transform::IDENTITY
        };
        let child = graph.add_node(Some(parent), child_local);

        assert_mat4_near(
            graph.world_transform(child),
            Transform {
                rotation: Quat::from_axis_angle(
                    Vec3::new(0.0, 0.0, 1.0),
                    std::f32::consts::FRAC_PI_2,
                ),
                ..Transform::IDENTITY
            }
            .to_mat4()
            .multiply(child_local.to_mat4()),
        );
    }

    #[test]
    fn sibling_world_transforms_are_isolated() {
        let mut graph = SceneGraph::new();
        let first = graph.add_node(
            None,
            Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                ..Transform::IDENTITY
            },
        );
        let second = graph.add_node(
            None,
            Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                ..Transform::IDENTITY
            },
        );

        graph.set_local_transform(
            first,
            Transform {
                translation: Vec3::new(3.0, 0.0, 0.0),
                ..Transform::IDENTITY
            },
        );

        assert_mat4_near(
            graph.world_transform(first),
            Mat4::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        );
        assert_mat4_near(
            graph.world_transform(second),
            Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)),
        );
    }
}
