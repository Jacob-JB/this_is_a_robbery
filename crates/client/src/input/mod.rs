use bevy::prelude::*;

pub fn build(app: &mut App) {
    app.init_resource::<ControlScheme>();
}

#[derive(Resource)]
pub struct ControlScheme {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub jump: KeyCode,

    pub mouse_sensitivity: Vec2,
}

impl Default for ControlScheme {
    fn default() -> Self {
        ControlScheme {
            move_forward: KeyCode::KeyW,
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyD,
            move_right: KeyCode::KeyA,
            jump: KeyCode::Space,

            mouse_sensitivity: Vec2::splat(0.002),
        }
    }
}
