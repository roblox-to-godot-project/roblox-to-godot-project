#![allow(dead_code)]

#![feature(trait_upcasting)]
#![feature(ptr_metadata)]
#![feature(arbitrary_self_types)]

#![allow(internal_features)]
#![feature(core_intrinsics)]

#![feature(breakpoint)]


mod core;
mod instance;
mod userdata;

use godot::{classes::Engine, prelude::*};
use rustversion_detect::RUST_VERSION;

#[cfg(debug_assertions)]
macro_rules! godot_debug {
    ($fmt:literal $(, $args:expr)* $(,)?) => {
        godot::prelude::godot_print_rich!("[color=cyan]{}[/color]\n[color=gray]stack traceback:\n{}[/color]", 
            format!($fmt, $(, $args)*), 
            std::backtrace::Backtrace::force_capture()
        );
    };
    ($thing:expr) => {
        godot::prelude::godot_print_rich!("[color=cyan]{}[/color]\n[color=gray]stack traceback:\n{}[/color]", 
            format!("{} = {:?}", stringify!($thing), $thing), 
            std::backtrace::Backtrace::force_capture()
        );
    };
    (backtrace $thing:expr) => {
        godot::prelude::godot_print_rich!("[color=gray]stack traceback:\n{}[/color]", $thing);
    };
}
#[cfg(not(debug_assertions))]
macro_rules! godot_debug {
    ($fmt:literal $(, $args:expr)* $(,)?) => {};
    ($thing:expr) => {};
}
pub(crate) use godot_debug;


use core::RobloxVM;

struct RobloxToGodotProjectExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RobloxToGodotProjectExtension {
    fn min_level() -> InitLevel {
        InitLevel::Scene
    }

    fn on_level_init(level: InitLevel) {
        
        match level {
            InitLevel::Scene => {
                assert!({
                    let v = (*Engine::singleton()).get_copyright_info().at(0).get("name").unwrap();
                    let s = String::from(v.stringify());
                    s.starts_with("Godot")
                }, "incompatible gdextension api header"); // Make sure the header won't randomly break at runtime.
                godot_print!("Roblox To Godot Project v{} (Rust runtime v{}) by {}\n", env!("CARGO_PKG_VERSION"), RUST_VERSION, {
                    let authors: &'static str = env!("CARGO_PKG_AUTHORS");
                    authors.replace(":", ", ")
                });
                let mut roblox_vm = RobloxVM::new(None);
                let env = roblox_vm.get_mut().get_main_state().create_env_from_global().unwrap();
                roblox_vm.get_mut()
                    .get_main_state()
                    .compile_jit("test.lua", include_str!("test.lua"), env).unwrap()
                    .call::<()>(()).unwrap();
                
            }
            _ => ()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2+2;
        assert_eq!(result, 4);
    }
}
