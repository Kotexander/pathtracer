use super::{
    bounding_box::{self, BoundingBox},
    bytes::Bytes,
    sphere::Sphere,
    vector3::Vector3,
};

// https://www.ks.uiuc.edu/Research/vmd/projects/ece498/raytracing/GPU_BVHthesis.pdf

const ESCAPE: u32 = 0;
const OBJECT: u32 = 1;

#[derive(Clone, Copy, Debug)]
pub struct BVHNode {
    pub bbox: BoundingBox,

    pub index_type: u32,
    pub index: u32,
}
impl Bytes for BVHNode {
    fn bytes(&self) -> Vec<u8> {
        let mut b = vec![];
        let b8 = [0u8; 4];
        b.extend(bytemuck::bytes_of(&self.bbox.min));
        b.extend(b8);
        b.extend(bytemuck::bytes_of(&self.bbox.max));
        b.extend(b8);

        b.extend(bytemuck::bytes_of(&self.index_type));
        b.extend(bytemuck::bytes_of(&self.index));
        b.extend(b8);
        b.extend(b8);

        b
    }
}

#[derive(Clone, Debug)]
enum Node {
    Node(BVHTree),
    Object((u32, BoundingBox)),
}

#[derive(Clone, Debug)]
pub struct BVHTree {
    bbox: BoundingBox,
    left: Box<Node>,
    right: Box<Node>,
}
impl BVHTree {
    pub fn new(scene: Vec<Sphere>) -> Self {
        let scene: Vec<(u32, Sphere)> = scene
            .into_iter()
            .enumerate()
            .map(|us| (us.0 as u32, us.1))
            .collect();

        Self::new_interior(scene)
    }
    fn new_interior(mut scene: Vec<(u32, Sphere)>) -> Self {
        if scene.len() == 0 || scene.len() == 1 {
            panic!("Scene length can't be 0 or 1")
        } else if scene.len() == 2 {
            let lhs: BoundingBox = scene[0].1.into();
            let rhs: BoundingBox = scene[1].1.into();
            let left = Box::new(Node::Object((scene[0].0, lhs)));
            let right = Box::new(Node::Object((scene[1].0, rhs)));
            let bbox = bounding_box::combine(&lhs, &rhs);

            Self { bbox, left, right }
        } else if scene.len() == 3 {
            let obj = scene.pop().unwrap();

            let bbox: BoundingBox = obj.1.into();
            let left = Box::new(Node::Object((obj.0, bbox)));

            let right = Self::new_interior(scene);
            let bbox = bounding_box::combine(&bbox, &right.bbox);
            let right = Box::new(Node::Node(right));
            Self { bbox, left, right }
        } else {
            let min = Vector3::new(f32::MAX, f32::MAX, f32::MAX);
            let max = Vector3::new(f32::MIN, f32::MIN, f32::MIN);
            let mut bbox = BoundingBox { min, max };
            for (_, sphere) in &scene {
                bbox = bounding_box::combine(&bbox, &sphere.into());
            }
            let diff = max - min;

            if diff.x > diff.y && diff.x > diff.z {
                scene.sort_by(|a, b| a.1.pos.x.partial_cmp(&b.1.pos.x).unwrap());
            } else if diff.y > diff.z {
                scene.sort_by(|a, b| a.1.pos.y.partial_cmp(&b.1.pos.y).unwrap());
            } else {
                scene.sort_by(|a, b| a.1.pos.z.partial_cmp(&b.1.pos.z).unwrap());
            }

            let scene_other = scene.split_off(scene.len() / 2);

            let left = Self::new_interior(scene);
            let right = Self::new_interior(scene_other);
            let bbox = bounding_box::combine(&left.bbox, &right.bbox);

            let mut left = Box::new(Node::Node(left));
            let mut right = Box::new(Node::Node(right));

            if rand::random() {
                std::mem::swap(&mut left, &mut right);
            }
            Self { bbox, left, right }
        }
    }
}
pub fn flatten(bvh: BVHTree) -> Vec<BVHNode> {
    let mut result = vec![];

    flatten_interior(Node::Node(bvh), &mut result);

    result.into_iter().map(|n| n.unwrap()).collect()
}
fn flatten_interior(node: Node, result: &mut Vec<Option<BVHNode>>) {
    match node {
        Node::Node(n) => {
            let i = result.len();
            result.push(None);
            flatten_interior(*n.left, result);
            flatten_interior(*n.right, result);
            result[i] = Some(BVHNode {
                bbox: n.bbox,
                index_type: ESCAPE,
                index: result.len() as u32,
            });
        }
        Node::Object((i, o)) => result.push(Some(BVHNode {
            bbox: o.into(),
            index_type: OBJECT,
            index: i,
        })),
    }
}
