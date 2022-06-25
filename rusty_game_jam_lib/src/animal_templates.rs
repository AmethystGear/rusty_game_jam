use gdnative::core_types::{Vector2, Vector3};

use crate::animal::Animal;
use crate::animal::BodyPoint;
use crate::animal::Limb;

pub fn chicken() -> Animal {
    let texture_indices = [Some((0, 1.0)), None];
    let chicken_leg = vec![
        BodyPoint {
            dir: Vector2::new(0.2, -0.7),
            size: 0.1,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(-0.2, -0.7),
            size: 0.1,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(-0.3, 0.0),
            size: 0.1,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.1, 0.0),
            size: 0.0,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
    ];
    Animal::new(vec![
        BodyPoint {
            dir: Vector2::new(1.0, 0.0),
            size: 1.0,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.5, -1.0),
            size: 0.5,
            texture_indices,
            discontinuous: true,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.75, -0.2),
            size: 1.0,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.75, 0.2),
            size: 1.1,
            texture_indices,
            discontinuous: false,
            limbs: vec![
                Limb {
                    name: "front_leg".to_string(),
                    displacement: Vector3::new(0.3, 0.1, -1.0),
                    texture_displacement: 5,
                    body: chicken_leg.clone(),
                },
                Limb {
                    name: "back_leg".to_string(),
                    displacement: Vector3::new(0.3, 0.1, 1.0),
                    texture_displacement: 5,
                    body: chicken_leg.clone(),
                },
            ],
        },
        BodyPoint {
            dir: Vector2::new(0.75, 1.0),
            size: 1.0,
            texture_indices: [Some((0, 1.0)), None],
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(1.0, 1.0),
            size: 0.1,
            texture_indices: [Some((0, 1.0)), None],
            discontinuous: false,
            limbs: Vec::new(),
        },
    ])
}

pub fn turtle() -> Animal {
    let texture_indices = [Some((2, 1.0)), None];
    let turtle_leg = vec![
        BodyPoint {
            dir: Vector2::new(0.65, -0.65),
            size: 0.3,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(-0.65, -0.65),
            size: 0.3,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.1, 0.0),
            size: 0.3,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
    ];
    Animal::new(vec![
        BodyPoint {
            dir: Vector2::new(1.0, 0.0),
            size: 1.0,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(2.0, 0.5),
            size: 10.0 / 16.0,
            texture_indices,
            discontinuous: true,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(2.0, 0.5) * 0.001,
            size: 1.0,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(2.0, 0.5),
            size: 2.0,
            texture_indices,
            discontinuous: false,
            limbs: vec![
                Limb {
                    name: "front_leg_0".to_string(),
                    displacement: Vector3::new(0.3, -0.2, -2.0),
                    texture_displacement: 5,
                    body: turtle_leg.clone(),
                },
                Limb {
                    name: "back_leg_0".to_string(),
                    displacement: Vector3::new(0.3, -0.2, -1.0),
                    texture_displacement: 5,
                    body: turtle_leg.clone(),
                },
            ],
        },
        BodyPoint {
            dir: Vector2::new(2.0, -0.5),
            size: 3.0,
            texture_indices,
            discontinuous: false,
            limbs: vec![
                Limb {
                    name: "front_leg_1".to_string(),
                    displacement: Vector3::new(0.3, -0.5, -2.0),
                    texture_displacement: 5,
                    body: turtle_leg.clone(),
                },
                Limb {
                    name: "back_leg_1".to_string(),
                    displacement: Vector3::new(0.3, -0.5, -1.0),
                    texture_displacement: 5,
                    body: turtle_leg.clone(),
                },
            ],
        },
        BodyPoint {
            dir: Vector2::new(2.0, -0.5).normalized() * 0.001,
            size: 2.0,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new()
        },
        BodyPoint {
            dir: Vector2::new(1.0, -0.5),
            size: 0.5,
            texture_indices,
            discontinuous: true,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.001, 0.0),
            size: 0.0,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.001, 0.0),
            size: 0.0,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.001, 0.0),
            size: 0.0,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
    ])
}

pub fn fox() -> Animal {
    let texture_indices = [Some((1, 1.0)), None];
    let fox_leg = vec![
        BodyPoint {
            dir: Vector2::new(0.45, -0.45),
            size: 0.2,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(-0.65, -0.65),
            size: 0.1,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.1, 0.0),
            size: 0.1,
            texture_indices,
            discontinuous: false,
            limbs: Vec::new(),
        },
    ];
    Animal::new(vec![
        BodyPoint {
            dir: Vector2::new(1.0, 0.0),
            size: 1.0,
            texture_indices: [Some((1, 1.0)), None],
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.75, 0.0),
            size: 0.5,
            texture_indices: [Some((1, 1.0)), None],
            discontinuous: true,
            limbs: vec![
                Limb {
                    name: "front_leg_2".to_string(),
                    displacement: Vector3::new(0.3, 0.2, -2.0),
                    texture_displacement: 7,
                    body: fox_leg.clone(),
                },
                Limb {
                    name: "back_leg_2".to_string(),
                    displacement: Vector3::new(0.3, 0.2, -1.0),
                    texture_displacement: 7,
                    body: fox_leg.clone(),
                },
            ],
        },
        BodyPoint {
            dir: Vector2::new(0.75, 0.0),
            size: 0.7,
            texture_indices: [Some((1, 1.0)), None],
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(0.5, 0.0),
            size: 0.8,
            texture_indices: [Some((1, 1.0)), None],
            discontinuous: false,
            limbs: vec![
                Limb {
                    name: "front_leg_3".to_string(),
                    displacement: Vector3::new(0.3, -0.2, -2.0),
                    texture_displacement: 7,
                    body: fox_leg.clone(),
                },
                Limb {
                    name: "back_leg_3".to_string(),
                    displacement: Vector3::new(0.3, -0.2, -1.0),
                    texture_displacement: 7,
                    body: fox_leg.clone(),
                },
            ],
        },
        BodyPoint {
            dir: Vector2::new(0.5, 0.0),
            size: 0.7,
            texture_indices: [Some((1, 1.0)), None],
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(1.0, 0.0),
            size: 0.1,
            texture_indices: [Some((1, 1.0)), None],
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(1.0, 0.0),
            size: 1.0,
            texture_indices: [Some((1, 1.0)), None],
            discontinuous: false,
            limbs: Vec::new(),
        },
        BodyPoint {
            dir: Vector2::new(1.0, 0.0),
            size: 0.1,
            texture_indices: [Some((1, 1.0)), None],
            discontinuous: false,
            limbs: Vec::new(),
        },
    ])
}
