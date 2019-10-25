#[macro_use] extern crate core_lib;
extern crate core_ui;
extern crate specs;
extern crate piston_window;

mod window_system;
mod utilities;

use piston_window::*;
pub use window_system::WindowSystem as PistonWindowSystem;

pub fn new_piston_window() -> PistonWindowSystem {
  let window : PistonWindow = WindowSettings::new("Geometry Sketchpad", core_lib::resources::WINDOW_SIZE).build().unwrap();
  window_system::WindowSystem { window }
}