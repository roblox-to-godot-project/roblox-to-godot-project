#![allow(dead_code)]

#![feature(trait_upcasting)]
#![feature(ptr_metadata)]

#![allow(internal_features)]
#![feature(core_intrinsics)]

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
                godot_print!("Roblox To Godot Project v{} (rust runtime v{}) by {}\n", env!("CARGO_PKG_VERSION"), RUST_VERSION, {
                    let authors: &'static str = env!("CARGO_PKG_AUTHORS");
                    authors.replace(":", ", ")
                });
                let mut roblox_vm = RobloxVM::new();
                let env = roblox_vm.get_mut().unwrap().get_main_state().get_lua().globals();
                roblox_vm.get_mut().unwrap()
                    .get_main_state()
                    .compile_jit("meow?", r#"
                        print("Hey there!", _VERSION, "running in Godot!")
                        warn("uwu")
                        print(Vector3int16.new(32768, 10, -11))
                        print(Instance)
                        print(Instance.new("Model"))
                    "#, env).unwrap()
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
