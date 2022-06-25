use std::collections::HashMap;
use std::ops::Div;

use gdnative::api::*;
use gdnative::prelude::*;
use itertools::Itertools;
use rand::thread_rng;
use rand::Rng;
use std::iter::zip;

trait Target {
    fn update_target(&mut self, target: Vector2);
    fn delta(&mut self, delta: f32);
    fn target(&self) -> Vector2;
    fn elapsed(&self) -> f32;
}

pub struct LinearTarget {
    prev: Vector2,
    curr: Vector2,
    elapsed: f32,
    time: f32,
}

impl LinearTarget {
    pub fn new(target: Vector2, time: f32) -> Self {
        Self {
            prev: target,
            curr: target,
            elapsed: f32::MAX,
            time,
        }
    }
}

impl Target for LinearTarget {
    fn update_target(&mut self, target: Vector2) {
        self.prev = self.curr;
        self.curr = target;
        self.elapsed = 0.0;
    }

    fn delta(&mut self, delta: f32) {
        self.elapsed += delta;
    }

    fn target(&self) -> Vector2 {
        if self.elapsed > self.time {
            self.curr
        } else {
            self.prev
                .linear_interpolate(self.curr, self.elapsed / self.time)
        }
    }

    fn elapsed(&self) -> f32 {
        self.elapsed
    }
}

pub struct ParabolicTarget {
    prev: Vector2,
    curr: Vector2,
    elapsed: f32,
    arc_time: f32,
    arc_slope: f32,
}

impl ParabolicTarget {
    pub fn new(target: Vector2, arc_time: f32, arc_slope: f32) -> Self {
        Self {
            prev: target,
            curr: target,
            elapsed: f32::MAX,
            arc_time,
            arc_slope,
        }
    }
}

impl Target for ParabolicTarget {
    fn update_target(&mut self, target: Vector2) {
        self.prev = self.curr;
        self.curr = target;
        self.elapsed = 0.0;
    }

    fn delta(&mut self, delta: f32) {
        self.elapsed += delta;
    }

    fn target(&self) -> Vector2 {
        if self.elapsed > self.arc_time {
            self.curr
        } else {
            let diff = self.curr.x - self.prev.x;
            let x = self.elapsed / self.arc_time * diff;
            Vector2::new(
                x,
                -self.arc_slope / diff.abs() * x * x + x.abs() * self.arc_slope,
            ) + self.prev
        }
    }

    fn elapsed(&self) -> f32 {
        self.elapsed
    }
}

#[derive(NativeClass)]
#[inherit(RigidBody)]
pub struct AnimalController {
    targets: Vec<(String, String, usize, usize, Box<dyn Target>)>,
    bones: HashMap<String, Vec<i64>>,
    time: f64,
    limb_length: f32,
    animal_dimensions: Vector2,
    random_next: f64,
}

const NUM_IK_ITERATIONS: usize = 10;
#[methods]
impl AnimalController {
    fn new(_: &RigidBody) -> Self {
        Self {
            targets: Vec::new(),
            bones: HashMap::new(),
            limb_length: 0.0,
            animal_dimensions: Vector2::ZERO,
            time: 0.0,
            random_next: 0.0,
        }
    }

    #[export]
    fn _ready(&mut self, owner: &RigidBody) {
        self.ready(owner).unwrap();
    }

    fn ready(&mut self, owner: &RigidBody) -> Option<()> {
        let skeleton = unsafe {
            owner
                .get_child(1)?
                .assume_safe()
                .get_child(0)?
                .assume_safe()
                .cast::<Skeleton>()?
        };

        let mut names: Vec<_> = ["front_leg", "back_leg"]
            .iter()
            .flat_map(|x| {
                ["", "_0", "_1", "_2", "_3"]
                    .iter()
                    .map(move |y| format!("{}{}", x, y))
            })
            .collect();

        names.push("spine".to_string());

        for name in &names {
            let mut ids = Vec::new();
            while let Some(bone_id) = get_bone_id(&skeleton, &format!("{}_{}", name, ids.len())) {
                ids.push(bone_id);
            }
            self.bones.insert(name.to_string(), ids);
        }

        for name in &names {
            if name.contains("leg") && self.bones[name].len() > 0 {
                self.limb_length = get_limb_length(self.bones["front_leg"].clone(), &skeleton);
                break;
            }
        }

        for name in &names {
            if name.contains("leg") && self.bones[name].len() > 0 {
                if name.contains("front") {
                    let front_leg_target =
                        vec2(global_posn(&skeleton, self.bones[name][0])) + Vector2::new(0.4, -1.5);
                    self.targets.push((
                        name.clone(),
                        "leg".into(),
                        0,
                        self.bones[name].len() - 1,
                        Box::new(ParabolicTarget::new(front_leg_target, 0.15, 0.2)),
                    ));
                } else if name.contains("back") {
                    let back_leg_target = vec2(global_posn(&skeleton, self.bones[name][0]))
                        + Vector2::new(-0.4, -1.5);
                    self.targets.push((
                        name.into(),
                        "leg".into(),
                        0,
                        self.bones[name].len() - 1,
                        Box::new(ParabolicTarget::new(back_leg_target, 0.15, 0.2)),
                    ));
                }
            }
        }

        let tail_target = vec2(global_posn(
            &skeleton,
            self.bones["spine"][self.bones["spine"].len() - 1],
        ));
        self.targets.push((
            "spine".into(),
            "tail".into(),
            self.bones["spine"].len() - 3,
            self.bones["spine"].len() - 1,
            Box::new(LinearTarget::new(tail_target, 1.0)),
        ));

        Some(())
    }

    #[export]
    fn _process(&mut self, owner: &RigidBody, delta: f64) {
        self.process(owner, delta as f32).unwrap();
        self.time += delta;
        //owner.add_central_force(Vector3::new(-10.0, 0.0, 0.0));
    }

    fn process(&mut self, owner: &RigidBody, delta: f32) -> Option<()> {
        let skeleton = unsafe {
            owner
                .get_child(1)?
                .assume_safe()
                .get_child(0)?
                .assume_safe()
                .cast::<Skeleton>()?
        };
        let mut names: Vec<_> = ["front_leg", "back_leg"]
            .iter()
            .flat_map(|x| {
                ["", "_0", "_1", "_2", "_3"]
                    .iter()
                    .map(move |y| format!("{}{}", x, y))
            })
            .collect();

        names.push("spine".to_string());

        let mut bones = HashMap::new();
        for name in names {
            let mut ids = Vec::new();
            while let Some(bone_id) = get_bone_id(&skeleton, &format!("{}_{}", name, ids.len())) {
                ids.push(bone_id);
            }
            bones.insert(name.to_string(), ids);
        }

        let mut reached_end = Vec::new();
        for (limb_name, _, start_bone, end_bone, target) in &self.targets {
            reached_end.push(inverse_kinematics(
                &skeleton,
                &bones,
                limb_name,
                target.target(),
                *start_bone,
                *end_bone,
            ));
        }

        for (i, (limb_name, limb_type, start_bone, end_bone, target)) in
            self.targets.iter_mut().enumerate()
        {
            let limb_root_posn = vec2(global_posn(&skeleton, self.bones[limb_name][*start_bone]));
            if !reached_end[i] && limb_type == "leg" {
                let x_diff = self.animal_dimensions.x * 0.3;
                let y = (self.limb_length * self.limb_length - x_diff * x_diff).sqrt();
                //let mut rng = rand::thread_rng();
                //let r = rng.gen::<f32>() - 0.5;
                if target.target().x < limb_root_posn.x {
                    target.update_target(limb_root_posn + Vector2::new(0.8, -1.5));
                } else {
                    target.update_target(limb_root_posn + Vector2::new(-0.8, -1.5));
                }
            }
            if limb_type == "tail" {
                target.update_target(
                    vec2(global_posn(&skeleton, self.bones[limb_name][*start_bone]))
                        + Vector2::new(5.0, self.time.sin() as f32),
                );

                /*
                if target.elapsed() > 1.5 {
                    let mut rng = rand::thread_rng();
                    let pose = skeleton.get_bone_pose(self.bones[limb_name][*start_bone]);
                    skeleton.set_bone_pose(self.bones[limb_name][*start_bone], rotate(0.0));


                    let angle  = rng.gen::<f32>() - 0.5 + 3.1415;
                    target.update_target(
                        vec2(global_posn(&skeleton, self.bones[limb_name][*start_bone]))
                            - Vector2::new(angle.cos(), angle.sin()) * 2.0,
                    );
                    skeleton.set_bone_pose(self.bones[limb_name][*start_bone], pose);
                }
                */
            }
            target.delta(delta);
        }

        if self.time > self.random_next {
            let mut rng = rand::thread_rng();
            owner.set_axis_velocity(Vector3::new((rng.gen::<f32>() - 0.5) * 12.0, 0.0, 0.0));
            self.random_next = self.time + rng.gen::<f64>() * 5.0;
        }

        Some(())
    }
}

fn rotate(angle: f32) -> Transform {
    Transform {
        basis: Basis::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), angle),
        origin: Vector3::ZERO,
    }
}

fn global_posn(skeleton: &Skeleton, bone_idx: i64) -> Vector3 {
    (skeleton.global_transform() * skeleton.get_bone_global_pose(bone_idx)).origin
}

fn get_limb_length(limb: Vec<i64>, skeleton: &Skeleton) -> f32 {
    let mut length = 0.0;
    for bone_idx in &limb {
        let diff = skeleton.get_bone_rest(*bone_idx).origin;
        length += diff.distance_to(Vector3::ZERO);
    }
    length
}

fn inverse_kinematics(
    skeleton: &TRef<Skeleton>,
    bones: &HashMap<String, Vec<i64>>,
    limb_name: &str,
    target: Vector2,
    start_bone: usize,
    end_bone: usize,
) -> bool {
    let limb = bones[limb_name].clone();
    let limb_posn =
        (skeleton.global_transform() * skeleton.get_bone_global_pose(limb[start_bone])).origin;
    let target = Vector2::new(target.x - limb_posn.x, target.y - limb_posn.y);

    let mut bone_positions = Vec::new();
    let mut current_posn = Vector3::ZERO;
    bone_positions.push(current_posn);
    for bone_idx in &limb[(start_bone + 1)..(end_bone + 1)] {
        current_posn += skeleton.get_bone_rest(*bone_idx).origin;
        bone_positions.push(current_posn);
    }

    let mut ik_bone_positions = bone_positions.clone();
    for _ in 0..NUM_IK_ITERATIONS {
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

    let mut curr_angle = (skeleton.global_transform()
        * skeleton.get_bone_global_pose(skeleton.get_bone_parent(limb[start_bone])))
    .basis
    .to_euler()
    .z;

    for (i, ((rest_start, rest_end), (pose_start, pose_end))) in zip(
        bone_positions.iter().tuple_windows(),
        ik_bone_positions.iter().tuple_windows(),
    )
    .enumerate()
    {
        /*
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
        */

        let rest = vec2(*rest_end - *rest_start);
        let pose = vec2(*pose_end - *pose_start);
        let angle = -pose.angle() + rest.angle();
        skeleton.set_bone_pose(limb[i + start_bone], rotate(angle - curr_angle));
        curr_angle = angle;
    }

    vec2(ik_bone_positions[ik_bone_positions.len() - 1]).distance_squared_to(target) < 0.01
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
