use specs::prelude::*;

pub use crate::{
  util::Key,
  resources::{InputState, Tool},
  systems::events::{ToolChangeEventChannel, ToolChangeEvent},
};

pub struct ChangeToolViaKeyboard;

impl<'a> System<'a> for ChangeToolViaKeyboard {
  type SystemData = (
    Read<'a, InputState>,
    Write<'a, ToolChangeEventChannel>,
  );

  fn run(&mut self, (input_state, mut tool_change_events): Self::SystemData) {
    if input_state.keyboard.just_activated(Key::S) {
      tool_change_events.single_write(ToolChangeEvent(Tool::Select));
    } else if input_state.keyboard.just_activated(Key::P) {
      tool_change_events.single_write(ToolChangeEvent(Tool::Point));
    } else if input_state.keyboard.just_activated(Key::L) {
      tool_change_events.single_write(ToolChangeEvent(Tool::Line));
    } else if input_state.keyboard.just_activated(Key::C) {
      tool_change_events.single_write(ToolChangeEvent(Tool::Circle));
    }
  }
}