mod animal;
mod animal_spawner;
mod animal_controller;
mod animal_templates;
mod free_cam;
mod prop_ref;


use animal_controller::AnimalController;
use animal_spawner::AnimalSpawner;
use backtrace::Backtrace;
use free_cam::FreeCam;
use gdnative::prelude::*;

// lets us catch panics and report them to godot
fn init_panic_hook() {
    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let loc_string = if let Some(location) = panic_info.location() {
            format!("file '{}' at line {}", location.file(), location.line())
        } else {
            "unknown location".into()
        };

        let error_message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            format!("[RUST] {}: panic occurred: {:?}", loc_string, s)
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            format!("[RUST] {}: panic occurred: {:?}", loc_string, s)
        } else {
            format!("[RUST] {}: unknown panic occurred", loc_string)
        };
        let error_message = format!("{}\n{:?}", error_message, Backtrace::new());

        (*(old_hook.as_ref()))(panic_info);

        unsafe {
            if let Some(gd_panic_hook) = autoload::<Node>("rust_panic_hook") {
                gd_panic_hook.call(
                    "rust_panic_hook",
                    &[GodotString::from_str(error_message).to_variant()],
                );
            }
        }
    }));
}

fn init(handle: InitHandle) {
    handle.add_class::<FreeCam>();
    handle.add_class::<AnimalSpawner>();
    handle.add_class::<AnimalController>();
    init_panic_hook();
}

godot_init!(init);

#[cfg(test)]
mod tests {
    use crate::animal::{blend_animals, Animal, BodyGradient, BodyPoint};
    use gdnative::prelude::*;

    #[test]
    fn test_animal_blend() {
        let animal_textures = [[Some((0, 1.0)), None], [Some((1, 1.0)), None]];
        let animals = [
            Animal::new(vec![
                BodyPoint {
                    dir: Vector2::new(1.0, -1.0),
                    size: 1.0,
                    texture_indices: animal_textures[0],
                    discontinuous: false,
                    limbs: Vec::new(),
                },
                BodyPoint {
                    dir: Vector2::new(1.0, -0.2),
                    size: 1.0,
                    texture_indices: animal_textures[0],
                    discontinuous: false,
                    limbs: Vec::new(),
                },
            ]),
            Animal::new(vec![
                BodyPoint {
                    dir: Vector2::new(-1.0, 1.0),
                    size: 3.0,
                    texture_indices: animal_textures[1],
                    discontinuous: false,
                    limbs: Vec::new(),
                },
                BodyPoint {
                    dir: Vector2::new(-1.0, 0.2),
                    size: 3.0,
                    texture_indices: animal_textures[1],
                    discontinuous: false,
                    limbs: Vec::new(),
                },
            ]),
        ];
        let gradients = [
            BodyGradient(vec![0.25, 0.75]),
            BodyGradient(vec![0.75, 0.25]),
        ];

        let combined = animals.into_iter().zip(gradients).collect::<Vec<_>>();

        let blended = blend_animals(&combined);

        assert_eq!(
            blended,
            Animal::new(vec![
                BodyPoint {
                    dir: Vector2 { x: -0.5, y: 0.5 },
                    size: 2.5,
                    texture_indices: [Some((1, 0.75)), Some((0, 0.25))],
                    discontinuous: false,
                    limbs: Vec::new()
                },
                BodyPoint {
                    dir: Vector2 {
                        x: 0.5,
                        y: -0.10000001
                    },
                    size: 1.5,
                    texture_indices: [Some((0, 0.75)), Some((1, 0.25))],
                    discontinuous: false,
                    limbs: Vec::new()
                }
            ])
        );
    }
}
