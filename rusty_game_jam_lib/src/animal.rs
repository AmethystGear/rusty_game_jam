use crate::{animal_controller::AnimalController, animal_spawner::AnimalSpawner};
use gdnative::{api::*, prelude::*};
use itertools::Itertools;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub struct Animal {
    body: Limb,
}

impl Animal {
    pub fn new(body: Vec<BodyPoint>) -> Self {
        Self {
            body: Limb {
                body,
                texture_displacement: 0,
                displacement: Vector3::ZERO,
                name: "spine".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Limb {
    pub displacement: Vector3,
    pub texture_displacement: usize,
    pub body: Vec<BodyPoint>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BodyPoint {
    pub dir: Vector2,
    pub size: f32,
    pub texture_indices: [Option<(usize, f32)>; 2],
    pub discontinuous: bool,
    pub limbs: Vec<Limb>,
}

pub struct BodyGradient(pub Vec<f32>);
impl BodyGradient {
    pub fn decreasing_linear(n: usize) -> Self {
        let mut v = Vec::new();
        for i in 0..n {
            v.push(((n.saturating_sub(i + 1)) as f32) / ((n - 1) as f32));
        }
        Self(v)
    }

    pub fn increasing_linear(n: usize) -> Self {
        let mut v = Vec::new();
        for i in 0..n {
            v.push((i.saturating_sub(1) as f32) / ((n - 1) as f32));
        }
        Self(v)
    }

    pub fn fill_zeroes_till(mut self, n: usize) -> Self {
        while self.0.len() < n {
            self.0.push(0.0);
        }
        self
    }

    pub fn start_with(mut self, vals: Vec<f32>) -> Self {
        for i in 0..vals.len() {
            self.0[i] = vals[i];
        }
        self
    }
}

pub fn blend_animals(animals: &[(Animal, BodyGradient)]) -> Animal {
    let float_compare = |a: &f32, b: &f32| a.partial_cmp(b).unwrap_or(Ordering::Less);
    let body_len = animals
        .iter()
        .map(|(x, _)| x.body.body.len())
        .max()
        .expect("can't blend 0 animals together");

    let mut body: Vec<BodyPoint> = Vec::new();
    for i in 0..body_len {
        let mut dir = Vector2::ZERO;
        let mut size = 0.0;
        let mut texture_indices = Vec::new();
        let mut grad_sum = 0.0;
        let mut discontinuous = false;
        let mut limbs = Vec::new();
        let mut max_grad = f32::MIN;
        for (animal, body_grad) in animals {
            if let Some(body_point) = animal.body.body.get(i) {
                let grad = *body_grad.0.get(i).unwrap_or(&0f32);
                dir += body_point.dir * grad;
                size += body_point.size * grad;
                body_point.texture_indices.iter().for_each(|x| {
                    if let Some(x) = x {
                        texture_indices.push((x.0, x.1 * grad));
                    }
                });
                if body_point.discontinuous {
                    discontinuous = true;
                }
                grad_sum += grad;

                if body_point.limbs.len() > 0 && grad > max_grad {
                    limbs = body_point.limbs.clone();
                    max_grad = grad;
                }
            }
        }
        size /= grad_sum;
        dir /= grad_sum;

        let tex_compare = |a: &(usize, f32), b: &(usize, f32)| float_compare(&a.1, &b.1);
        texture_indices.sort_by(tex_compare);

        let first_tex_index = texture_indices[texture_indices.len() - 1];
        let second_tex_index = if let Some(body_point) = body.get(body.len().saturating_sub(1)) {
            let prev_contains = |tex_index: &(usize, f32)| {
                body_point
                    .texture_indices
                    .iter()
                    .any(|x| x.iter().any(|x| x.0 == tex_index.0))
            };
            if !prev_contains(&first_tex_index) {
                texture_indices
                    .into_iter()
                    .filter(|x| {
                        body_point
                            .texture_indices
                            .iter()
                            .any(|y| y.iter().any(|y| x.0 == y.0))
                    })
                    .max_by(|a, b| float_compare(&a.1, &b.1))
            } else if texture_indices.len() == 1
                || !prev_contains(&texture_indices[texture_indices.len() - 2])
            {
                None
            } else {
                Some(texture_indices[texture_indices.len() - 2])
            }
        } else {
            Some(texture_indices[texture_indices.len() - 2])
        };

        let texture_indices = [Some(first_tex_index), second_tex_index];

        body.push(BodyPoint {
            dir,
            size,
            texture_indices,
            discontinuous,
            limbs,
        });
    }

    // normalize body weights
    for body_point in &mut body {
        let total = body_point
            .texture_indices
            .iter()
            .map(|x| x.iter().map(|x| x.1).sum::<f32>())
            .sum::<f32>();

        for texture_index in &mut body_point.texture_indices {
            if let Some(texture_index) = texture_index {
                texture_index.1 /= total;
            }
        }
    }
    Animal::new(body)
}

fn get_uv(texture_block_size: (f32, f32), x: usize, y: usize, uv: (f32, f32)) -> Vector2 {
    Vector2::new(
        (x as f32 + uv.0) * texture_block_size.0,
        (y as f32 + uv.1) * texture_block_size.1,
    )
}

fn get_bone_id(skeleton: &Ref<Skeleton, Unique>, name: &str) -> i64 {
    for i in 0..skeleton.get_bone_count() {
        if skeleton.get_bone_name(i).to_string() == name {
            return i;
        }
    }
    unreachable!("bone {} does not exist", name)
}

fn create_animal_skeleton(animal: &Animal) -> Ref<Skeleton, Unique> {
    let skeleton = Skeleton::new();
    let mut limbs = vec![(&animal.body, None)];
    while let Some((limb, parent)) = limbs.pop() {
        create_skeleton_limb(limb, &skeleton, parent);
        for (i, point) in limb.body.iter().enumerate() {
            for new_limb in &point.limbs {
                limbs.push((new_limb, Some(format!("{}_{}", limb.name, i))));
            }
        }
    }
    skeleton
}

pub fn create_skeleton_limb(
    limb: &Limb,
    skeleton: &Ref<Skeleton, Unique>,
    parent_bone_name: Option<String>,
) {
    let mut last_dir = Vector2::new(limb.displacement.x, limb.displacement.y);
    for (i, point) in limb.body.iter().enumerate() {
        let bone_name = format!("{}_{}", limb.name, i);
        skeleton.add_bone(bone_name.clone());
        let bone_transform =
            Transform::IDENTITY.translated(Vector3::new(last_dir.x, last_dir.y, 0.0));
        skeleton.set_bone_rest(get_bone_id(&skeleton, &bone_name), bone_transform);
        if i != 0 {
            skeleton.set_bone_parent(
                get_bone_id(&skeleton, &bone_name),
                get_bone_id(&skeleton, &format!("{}_{}", limb.name, i - 1)),
            );
        } else if let Some(parent_bone_name) = &parent_bone_name {
            skeleton.set_bone_parent(
                get_bone_id(skeleton, &bone_name),
                get_bone_id(skeleton, parent_bone_name),
            );
        }
        last_dir = point.dir;
    }
}

const COMPRESS_FLAGS_DEFAULT: i64 = 97280;
pub fn create_limb_mesh(
    limb: &Limb,
    current_posn: Vector3,
    skeleton: &Ref<Skeleton, Unique>,
    animal_material: &Ref<ShaderMaterial>,
    texture_block_size: (f32, f32),
) -> Ref<MeshInstance, Unique> {
    let mesh = ArrayMesh::new();
    let st = SurfaceTool::new();
    st.begin(Mesh::PRIMITIVE_TRIANGLES);

    let mut current_posn = current_posn;

    let mut last_dir = limb.body[0].dir;
    let average_dir = |a: Vector2, b: Vector2| (a + b) / 2.0;
    for ((i, first), (_, second)) in limb.body.iter().enumerate().tuple_windows() {
        let diff_first = if first.discontinuous {
            last_dir.tangent().normalized() * first.size * 0.5
        } else {
            average_dir(first.dir, last_dir).tangent().normalized() * first.size * 0.5
        };

        let diff_second = if second.discontinuous {
            diff_first
        } else {
            average_dir(second.dir, first.dir).tangent().normalized() * second.size * 0.5
        };
        last_dir = first.dir;

        let [diff_first, diff_second, first_dir] =
            [diff_first, diff_second, first.dir].map(|x| Vector3::new(x.x, x.y, 0.0));

        let corners = [
            (current_posn - diff_first, (0.0, 0.0)),
            (current_posn + diff_first, (0.0, 1.0)),
            (current_posn + diff_second + first_dir, (1.0, 1.0)),
            (current_posn - diff_second + first_dir, (1.0, 0.0)),
        ];
        let center = (current_posn + first_dir * 0.5, (0.5, 0.5));
        let quad = [
            [corners[0], corners[1], center],
            [corners[1], corners[2], center],
            [corners[2], corners[3], center],
            [corners[3], corners[0], center],
        ];

        for (index, tri) in quad.into_iter().enumerate() {
            let (first_texture_indices, second_texture_indices) =
                match (first.texture_indices, second.texture_indices) {
                    ([Some(a), None], [Some(b), None]) => {
                        if a.0 != b.0 {
                            unreachable!()
                        }
                        (vec![a], vec![b])
                    }
                    ([Some(a), None], [Some(b), Some(c)]) => {
                        if a.0 == b.0 {
                            (vec![a, (c.0, 0.0)], vec![b, c])
                        } else if a.0 == c.0 {
                            (vec![(b.0, 0.0), a], vec![b, c])
                        } else {
                            unreachable!()
                        }
                    }
                    ([Some(a), Some(b)], [Some(c), None]) => {
                        if c.0 == a.0 {
                            (vec![a, b], vec![c, (b.0, 0.0)])
                        } else if c.0 == b.0 {
                            (vec![a, b], vec![(a.0, 0.0), c])
                        } else {
                            unreachable!()
                        }
                    }
                    ([Some(a), Some(b)], [Some(c), Some(d)]) => {
                        if a.0 == c.0 && b.0 == d.0 {
                            (vec![a, b], vec![c, d])
                        } else if a.0 == d.0 && b.0 == c.0 {
                            (vec![b, a], vec![c, d])
                        } else {
                            unreachable!()
                        }
                    }
                    _ => unreachable!(),
                };

            for (vert, uv) in tri {
                let uvs = first_texture_indices
                    .iter()
                    .map(|tex| get_uv(texture_block_size, i + limb.texture_displacement, tex.0, uv))
                    .collect_vec();

                if uvs.len() == 1 {
                    st.add_uv(uvs[0]);
                    st.add_color(Color::from_rgba(0.0, 0.0, 1.0, index as f32));
                } else if uvs.len() == 2 {
                    st.add_uv(uvs[0]);
                    let alphas = if uv.0 == 0.0 {
                        &first_texture_indices
                    } else {
                        &second_texture_indices
                    }
                    .iter()
                    .map(|x| x.1)
                    .collect_vec();
                    st.add_color(Color::from_rgba(
                        uvs[1].x,
                        uvs[1].y,
                        alphas[0],
                        index as f32,
                    ))
                }

                let mut bones = PoolArray::new();
                let mut weights = PoolArray::new();

                let [first_bone, second_bone] = [i, i + 1]
                    .map(|x| get_bone_id(&skeleton, &format!("{}_{}", limb.name, x)) as i32);

                if uv.0 < 0.25 {
                    bones.push(first_bone);
                    weights.push(1.0);
                } else if uv.0 > 0.75 {
                    bones.push(second_bone);
                    weights.push(1.0);
                } else {
                    bones.push(first_bone);
                    bones.push(second_bone);
                    weights.push(0.5);
                    weights.push(0.5);
                }

                while bones.len() < 4 {
                    bones.push(0);
                    weights.push(0.0);
                }
                st.add_bones(bones);
                st.add_weights(weights);

                st.add_vertex(vert);
            }
        }

        current_posn += first_dir;
    }

    mesh.add_surface_from_arrays(
        Mesh::PRIMITIVE_TRIANGLES,
        st.commit_to_arrays(),
        VariantArray::new_shared(),
        COMPRESS_FLAGS_DEFAULT,
    );

    let limb_mesh = MeshInstance::new();
    limb_mesh.set_mesh(mesh);
    limb_mesh.set_material_override(animal_material);
    limb_mesh
}

pub fn create_animal_meshes(
    animal: &Animal,
    skeleton: &Ref<Skeleton, Unique>,
    animal_material: &Ref<ShaderMaterial>,
    texture_block_size: (f32, f32),
) -> Vec<Ref<MeshInstance, Unique>> {
    let mut meshes = Vec::new();
    let mut limbs = vec![(&animal.body, Vector3::ZERO)];
    while let Some((limb, current_posn)) = limbs.pop() {
        let mesh = create_limb_mesh(
            limb,
            current_posn + Vector3::new(limb.displacement.x, limb.displacement.y, 0.0),
            skeleton,
            animal_material,
            texture_block_size,
        );
        mesh.set_transform(Transform::IDENTITY.translated(Vector3::new(
            0.0,
            0.0,
            limb.displacement.z,
        )));
        meshes.push(mesh);
        let mut current_posn = current_posn;
        for point in &limb.body {
            for new_limb in &point.limbs {
                limbs.push((new_limb, current_posn));
            }
            current_posn += Vector3::new(point.dir.x, point.dir.y, 0.0);
        }
    }
    meshes
}

pub fn create_animal(
    animal: &Animal,
    script: &Ref<Script>,
    animal_material: &Ref<ShaderMaterial>,
    texture_block_size: (f32, f32),
) -> Ref<Spatial, Unique> {
    let animal_node = Spatial::new();
    let animal_skeleton = create_animal_skeleton(animal);
    let animal_meshes = create_animal_meshes(
        animal,
        &animal_skeleton,
        animal_material,
        texture_block_size,
    );
    for animal_mesh in animal_meshes {
        animal_mesh.set_skeleton_path(NodePath::from_str(".."));
        animal_skeleton.add_child(animal_mesh, false);
    }
    animal_node.add_child(animal_skeleton, false);
    animal_node.set_script(script);
    animal_node
}

/*
pub fn generate_animal(
    animal: &Animal,
    animal_material: &Ref<ShaderMaterial>,
    texture_block_size: (f32, f32),
) -> (Ref<MeshInstance, Unique>, Ref<Skeleton, Unique>) {
    let skeleton = create_animal_skeleton(animal);

    let mesh = ArrayMesh::new();
    let st = SurfaceTool::new();
    st.begin(Mesh::PRIMITIVE_TRIANGLES);

    let mut current_posn = Vector2::ZERO;
    let mut last_dir = animal.body.body[0].dir;
    let average_dir = |a: Vector2, b: Vector2| (a + b) / 2.0;
    let mut limb = &animal.body;
    for ((i, first), (_, second)) in limb.body.iter().enumerate().tuple_windows() {
        let diff_first = if first.discontinuous {
            last_dir.tangent().normalized() * first.size * 0.5
        } else {
            average_dir(first.dir, last_dir).tangent().normalized() * first.size * 0.5
        };

        let diff_second = if second.discontinuous {
            diff_first
        } else {
            average_dir(second.dir, first.dir).tangent().normalized() * second.size * 0.5
        };
        last_dir = first.dir;

        let corners = [
            (current_posn - diff_first, (0.0, 0.0)),
            (current_posn + diff_first, (0.0, 1.0)),
            (current_posn + diff_second + first.dir, (1.0, 1.0)),
            (current_posn - diff_second + first.dir, (1.0, 0.0)),
        ];
        let center = (current_posn + first.dir * 0.5, (0.5, 0.5));
        let quad = [
            [corners[0], corners[1], center],
            [corners[1], corners[2], center],
            [corners[2], corners[3], center],
            [corners[3], corners[0], center],
        ];

        for (index, tri) in quad.into_iter().enumerate() {
            let (first_texture_indices, second_texture_indices) =
                match (first.texture_indices, second.texture_indices) {
                    ([Some(a), None], [Some(b), None]) => {
                        if a.0 != b.0 {
                            unreachable!()
                        }
                        (vec![a], vec![b])
                    }
                    ([Some(a), None], [Some(b), Some(c)]) => {
                        if a.0 == b.0 {
                            (vec![a, (c.0, 0.0)], vec![b, c])
                        } else if a.0 == c.0 {
                            (vec![(b.0, 0.0), a], vec![b, c])
                        } else {
                            unreachable!()
                        }
                    }
                    ([Some(a), Some(b)], [Some(c), None]) => {
                        if c.0 == a.0 {
                            (vec![a, b], vec![c, (b.0, 0.0)])
                        } else if c.0 == b.0 {
                            (vec![a, b], vec![(a.0, 0.0), c])
                        } else {
                            unreachable!()
                        }
                    }
                    ([Some(a), Some(b)], [Some(c), Some(d)]) => {
                        if a.0 == c.0 && b.0 == d.0 {
                            (vec![a, b], vec![c, d])
                        } else if a.0 == d.0 && b.0 == c.0 {
                            (vec![b, a], vec![c, d])
                        } else {
                            unreachable!()
                        }
                    }
                    _ => unreachable!(),
                };

            for (vert, uv) in tri {
                let uvs = first_texture_indices
                    .iter()
                    .map(|tex| get_uv(texture_block_size, i, tex.0, uv))
                    .collect_vec();

                if uvs.len() == 1 {
                    st.add_uv(uvs[0]);
                    st.add_color(Color::from_rgba(0.0, 0.0, 1.0, index as f32));
                } else if uvs.len() == 2 {
                    st.add_uv(uvs[0]);
                    let alphas = if uv.0 == 0.0 {
                        &first_texture_indices
                    } else {
                        &second_texture_indices
                    }
                    .iter()
                    .map(|x| x.1)
                    .collect_vec();
                    st.add_color(Color::from_rgba(
                        uvs[1].x,
                        uvs[1].y,
                        alphas[0],
                        index as f32,
                    ))
                }

                let mut bones = PoolArray::new();
                let mut weights = PoolArray::new();

                let [first_bone, second_bone] =
                    [i, i + 1].map(|x| get_bone_id(&skeleton, &format!("{}_{}", &x)) as i32);

                if uv.0 < 0.25 {
                    bones.push(first_bone);
                    weights.push(1.0);
                } else if uv.0 > 0.75 {
                    bones.push(second_bone);
                    weights.push(1.0);
                } else {
                    bones.push(first_bone);
                    bones.push(second_bone);
                    weights.push(0.5);
                    weights.push(0.5);
                }

                while bones.len() < 4 {
                    bones.push(0);
                    weights.push(0.0);
                }
                st.add_bones(bones);
                st.add_weights(weights);

                st.add_vertex(Vector3::new(vert.x, vert.y, 0.0));
            }
        }

        current_posn += first.dir;
    }

    mesh.add_surface_from_arrays(
        Mesh::PRIMITIVE_TRIANGLES,
        st.commit_to_arrays(),
        VariantArray::new_shared(),
        COMPRESS_FLAGS_DEFAULT,
    );

    let mesh_instance = MeshInstance::new();
    mesh_instance.set_mesh(mesh);
    mesh_instance.set_material_override(animal_material);
    (mesh_instance, skeleton)
}
*/

/*
#[derive(Clone)]
pub struct Animal {
    head_size: f32,
    segments: Vec<Segment>,
}

#[derive(Clone)]
pub struct Segment {
    segment: Vector2,

    start: SegmentProperties,
    end: SegmentProperties,
}

#[derive(Clone)]
pub struct TextureBlend {
    index: (usize, usize),
    blend_value: f32,
}

#[derive(Clone)]
pub struct SegmentProperties {
    size: f32,
    texture_blends: Vec<TextureBlend>,
}

pub struct BodyGradient(Vec<f32>);

impl BodyGradient {
    pub fn decreasing_linear(n: usize) -> Self {
        let v = Vec::new();
        for i in 0..n {
            v.push(((n - i) as f32) / ((n - 1) as f32));
        }
        Self(v)
    }

    pub fn increasing_linear(n: usize) -> Self {
        let v = Vec::new();
        for i in 0..n {
            v.push((i as f32) / ((n - 1) as f32));
        }
        Self(v)
    }
}

pub fn blend_animals(animals: &[(Animal, BodyGradient)]) -> Animal {
    let head_size = animals.iter().map(|(x, _)| x.head_size).sum() / (animals.len() as f32);
    let max_num_segments = animals
        .iter()
        .map(|(x, _)| x.segments.len())
        .max()
        .expect("can't blend 0 animals together");
    let mut segments = Vec::new();
    for i in 0..max_num_segments {
        let mut averaged_segment = Segment {
            segment: Vector2::ZERO,
            start: SegmentProperties {
                size: 0f32,
                texture_blends: Vec::new(),
            },
            end: SegmentProperties {
                size: 0f32,
                texture_blends: Vec::new(),
            },
        };
        let mut num_segments = 0;
        for (animal, segment_gradient) in animals {
            let total_contribution = 0f32;
            if let Some(segment) = animal.segments.get(i) {
                averaged_segment.segment += segment.segment * segment_gradient.0[i];
                let sum_segment_properties = |a: &mut SegmentProperties, b: &SegmentProperties| {
                    a.size += b.size * segment_gradient.0[i];
                    let texture_blends = b
                        .texture_blends
                        .iter()
                        .map(|x| TextureBlend {
                            index: x.index.clone(),
                            blend_value: x.blend_value * segment_gradient.0[i],
                        })
                        .collect();
                    a.texture_blends.append(&mut texture_blends);
                };
                sum_segment_properties(&mut averaged_segment.start, &segment.start);
                sum_segment_properties(&mut averaged_segment.end, &segment.end);
                total_contribution += segment_gradient.0[i];
            }
        }
        let fix_texture_blends = |texture_blends: &mut Vec<TextureBlend>| {
            a.texture_blends.sort_by(|a, b| {
                a.blend_value
                    .partial_cmp(&b.blend_value)
                    .unwrap_or(Ordering::Less)
            });


        };

        segments.push(averaged_segment);
    }

    Animal {
        head_size: (),
        segments: (),
    }
}
*/
