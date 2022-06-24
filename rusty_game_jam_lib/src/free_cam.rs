use gdnative::api::Camera2D;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Camera2D)]
pub struct FreeCam {
    #[property(default = 5.0)]
    move_speed: f32,
    #[property(default = 5.0)]
    move_speed_mul: f32,
}

#[methods]
impl FreeCam {
    fn new(_: &Camera2D) -> Self {
        Self {
            move_speed: 0.0,
            move_speed_mul: 0.0,
        }
    }

    #[export]
    fn _process(&mut self, owner: &Camera2D, delta: f32) {
        let position = owner.position();
        let mut movement = Vector2::new(0.0, 0.0);
        let input = Input::godot_singleton();
    
        if input.is_action_pressed("ui_up", false) {
            movement += Vector2::new(0.0, -1.0);
        }
        if input.is_action_pressed("ui_down", false) {
            movement += Vector2::new(0.0, 1.0);
        }
        if input.is_action_pressed("ui_left", false) {
            movement += Vector2::new(-1.0, 0.0);
        }
        if input.is_action_pressed("ui_right", false) {
            movement += Vector2::new(1.0, 0.0);
        }

        if input.is_action_just_released("wheel_down", false) {
            self.move_speed *= self.move_speed_mul
        }

        if input.is_action_just_released("wheel_up", false) {
            self.move_speed /= self.move_speed_mul
        }
        owner.set_position(position + movement * delta * self.move_speed);
    }
}
