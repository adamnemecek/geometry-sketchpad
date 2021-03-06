// Core crates
extern crate core_lib;
extern crate core_ui;
extern crate specs;

// Foundation library providing "new_piston_window"
extern crate geopad_foundation;

use core_ui::{resources::*, setup_core_ui};
use geopad_foundation::new_piston_window;
use specs::prelude::*;

fn main() {
  let mut world = World::new();
  let mut builder = DispatcherBuilder::new();

  // Setup the core ui
  setup_core_ui(&mut builder);

  // Add the window system and build the dispatcher
  builder.add_thread_local(new_piston_window());

  // Build the dispatcher
  let mut dispatcher = builder.build();
  dispatcher.setup(&mut world);
  while !world.fetch::<ExitState>().is_exiting() {
    dispatcher.dispatch(&mut world);
  }
}
