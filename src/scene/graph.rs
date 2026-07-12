use super::Transform;
use crate::math::Mat4;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(usize);

struct Node {
    local: Transform,
    parent: Option<NodeId>,
}

#[derive(Default)]
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
        let node = &self.nodes[id.0];
        let mut world = node.local.to_mat4();
        let mut parent = node.parent;

        while let Some(parent_id) = parent {
            let parent_node = &self.nodes[parent_id.0];
            world = parent_node.local.to_mat4().multiply(world);
            parent = parent_node.parent;
        }

        world
    }
}

#[cfg(test)]
mod tests {
    use crate::math::{Mat4, Quat, Vec3};
    use crate::scene::{SceneGraph, Transform};

    const EPSILON: f32 = 1.0e-6;

    fn assert_mat4_near(actual: Mat4, expected: Mat4) {
        for (actual, expected) in actual.data.iter().zip(expected.data.iter()) {
            assert!((actual - expected).abs() < EPSILON);
        }
    }

    #[test]
    fn root_node_world_transform_equals_local_transform() {
        let local = Transform {
            translation: Vec3::new(2.0, 3.0, 4.0),
            rotation: Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2),
            scale: Vec3::new(2.0, 1.0, 3.0),
        };
        let mut graph = SceneGraph::new();
        let root = graph.add_node(None, local);

        assert_mat4_near(graph.world_transform(root), local.to_mat4());
    }

    #[test]
    fn child_of_translated_parent_composes_translations() {
        let mut graph = SceneGraph::new();
        let parent = graph.add_node(
            None,
            Transform {
                translation: Vec3::new(2.0, 3.0, 0.0),
                ..Transform::IDENTITY
            },
        );
        let child = graph.add_node(
            Some(parent),
            Transform {
                translation: Vec3::new(4.0, -1.0, 0.0),
                ..Transform::IDENTITY
            },
        );

        assert_mat4_near(
            graph.world_transform(child),
            Mat4::from_translation(Vec3::new(6.0, 2.0, 0.0)),
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
        let child = graph.add_node(
            Some(parent),
            Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                ..Transform::IDENTITY
            },
        );

        assert_mat4_near(
            graph.world_transform(child),
            Transform {
                translation: Vec3::new(0.0, 1.0, 0.0),
                rotation: Quat::from_axis_angle(
                    Vec3::new(0.0, 0.0, 1.0),
                    std::f32::consts::FRAC_PI_2,
                ),
                ..Transform::IDENTITY
            }
            .to_mat4(),
        );
    }

    #[test]
    fn sibling_world_transforms_are_isolated() {
        let mut graph = SceneGraph::new();
        let parent = graph.add_node(
            None,
            Transform {
                translation: Vec3::new(10.0, 0.0, 0.0),
                ..Transform::IDENTITY
            },
        );
        let first_sibling = graph.add_node(
            Some(parent),
            Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                ..Transform::IDENTITY
            },
        );
        let second_sibling = graph.add_node(
            Some(parent),
            Transform {
                translation: Vec3::new(-2.0, 0.0, 0.0),
                ..Transform::IDENTITY
            },
        );
        let updated_first = Transform {
            translation: Vec3::new(4.0, 0.0, 0.0),
            ..Transform::IDENTITY
        };
        graph.set_local_transform(first_sibling, updated_first);

        assert_eq!(graph.local_transform(first_sibling), &updated_first);
        assert_mat4_near(
            graph.world_transform(first_sibling),
            Mat4::from_translation(Vec3::new(14.0, 0.0, 0.0)),
        );
        assert_mat4_near(
            graph.world_transform(second_sibling),
            Mat4::from_translation(Vec3::new(8.0, 0.0, 0.0)),
        );
    }
}
