use std::collections::HashMap;

use gdnative::api::*;
use gdnative::prelude::*;
use itertools::Itertools;
use std::iter::zip;

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct AnimalController {
    #[property]
    target: Vector2,
}

#[methods]
impl AnimalController {
    fn new(_: &Spatial) -> Self {
        Self {
            target: Vector2::ZERO,
        }
    }

    #[export]
    fn _process(&mut self, owner: &Spatial, _delta: f64) {
        self.process(owner);
    }

    fn process(&mut self, owner: &Spatial) -> Option<()> {
        let skeleton = unsafe { owner.get_child(0)?.assume_safe().cast::<Skeleton>()? };

        let mut bones = HashMap::new();
        for name in ["spine", "front_leg", "back_leg"] {
            let mut ids = Vec::new();
            while let Some(bone_id) = get_bone_id(&skeleton, &format!("{}_{}", name, ids.len())) {
                ids.push(bone_id);
            }
            bones.insert(name.to_string(), ids);
        }

        inverse_kinematics(owner, &skeleton, &bones, "front_leg", self.target, 100);

        inverse_kinematics(owner, &skeleton, &bones, "back_leg", self.target, 100);

        Some(())
    }
}

fn get_limb_position(owner: &Spatial, skeleton: &TRef<Skeleton>, limb_name: &str) -> Vector3 {
    let mut bone =
        get_bone_id(skeleton, &format!("{}_0", limb_name)).expect("limb name is invalid");

    let mut position = Vector3::ZERO;
    while bone != -1 {
        position += skeleton.get_bone_rest(bone).origin;
        bone = skeleton.get_bone_parent(bone);
    }
    position + owner.translation()
}

fn rotate(angle: f32) -> Transform {
    Transform {
        basis: Basis::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), angle),
        origin: Vector3::ZERO,
    }
}

fn inverse_kinematics(
    owner: &Spatial,
    skeleton: &TRef<Skeleton>,
    bones: &HashMap<String, Vec<i64>>,
    limb_name: &str,
    target: Vector2,
    iterations: usize,
) {
    let limb = bones[limb_name].clone();
    let limb_posn = (skeleton.global_transform() * skeleton.get_bone_global_pose(limb[0])).origin;
    let target = Vector2::new(target.x - limb_posn.x, target.y - limb_posn.y);

    let mut bone_positions = Vec::new();
    let mut current_posn = Vector3::ZERO;
    bone_positions.push(current_posn);
    for bone_idx in &limb[1..] {
        current_posn += skeleton.get_bone_rest(*bone_idx).origin;
        bone_positions.push(current_posn);
    }

    let mut ik_bone_positions = bone_positions.clone();
    for _ in 0..iterations {
        let last_index = ik_bone_positions.len() - 1;
        ik_bone_positions[last_index] = Vector3::new(target.x, target.y, 0.0);

        for i in (0..(ik_bone_positions.len() - 1)).rev() {
            let dist = bone_positions[i].distance_to(bone_positions[i + 1]);
            ik_bone_positions[i] = (ik_bone_positions[i] - ik_bone_positions[i + 1]).normalized()
                * dist
                + ik_bone_positions[i + 1];
        }

        ik_bone_positions[0] = Vector3::ZERO;

        for i in 1..ik_bone_positions.len() {
            let dist = bone_positions[i].distance_to(bone_positions[i - 1]);
            ik_bone_positions[i] = (ik_bone_positions[i] - ik_bone_positions[i - 1]).normalized()
                * dist
                + ik_bone_positions[i - 1];
        }
    }

    let mut curr_angle = 0.0;

    for (i, ((rest_start, rest_end), (pose_start, pose_end))) in zip(
        bone_positions.iter().tuple_windows(),
        ik_bone_positions.iter().tuple_windows(),
    )
    .enumerate()
    {
        unsafe {
            if let Some(debug_draw) = autoload::<Node>("DebugDraw") {
                debug_draw.call(
                    "draw_line_3d",
                    &[
                        (*pose_end + limb_posn + Vector3::new(0.0, 0.0, -2.0)).to_variant(),
                        (*pose_start + limb_posn + Vector3::new(0.0, 0.0, -2.0)).to_variant(),
                        Color::from_rgb(1.0, 0.0, 0.0).to_variant(),
                    ],
                );
            }
        }

        let rest = vec2(*rest_end - *rest_start);
        let pose = vec2(*pose_end - *pose_start);
        let angle = -pose.angle() + rest.angle();
        skeleton.set_bone_pose(bones[limb_name][i], rotate(angle - curr_angle));
        curr_angle = angle;
    }
}

fn get_bone_id(skeleton: &TRef<Skeleton>, name: &str) -> Option<i64> {
    for i in 0..skeleton.get_bone_count() {
        if skeleton.get_bone_name(i).to_string() == name {
            return Some(i);
        }
    }
    None
}

fn vec2(v: Vector3) -> Vector2 {
    Vector2::new(v.x, v.y)
}
