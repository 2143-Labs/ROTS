
pub fn kb_input(
    keyboard_input: Res<Input<KeyCode>>,
    config: Res<Config>,
) {
    if config.pressing_keybind(|x| keyboard_input.pressed(x), shared::GameAction::MoveBackward) {
        //Sokethig
    }
    if config.just_pressed(&keyboard_input, shared::GameAction::UnlockCursor) {
    }
}
