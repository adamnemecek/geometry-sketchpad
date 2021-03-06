use crate::resources::*;
use core_lib::events::*;
use specs::prelude::*;

#[derive(Default)]
pub struct CreatePerpendicularViaKeyboard;

impl<'a> System<'a> for CreatePerpendicularViaKeyboard {
  type SystemData = (Read<'a, InputState>, Write<'a, CommandEventChannel>);

  fn run(&mut self, (input_state, mut command_event_channel): Self::SystemData) {
    let cmd = input_state.keyboard.is_command_activated();
    let shift = input_state.keyboard.is_shift_activated();
    let backslash = input_state.keyboard.just_activated(Key::Backslash);
    if cmd && shift && backslash {
      command_event_channel.single_write(CommandEvent {
        command: Command::LineInsert(InsertLineEvent::InsertPerpendicularFromSelection),
        event_id: None,
      });
    }
  }
}
