#![allow(dead_code)]

mod core;
mod instance;
mod userdata;

use godot::{classes::Engine, prelude::*};
use mlua::prelude::*;

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
                godot_print!("Roblox To Godot Project v{} by {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
                let mut roblox_vm = RobloxVM::new();
                let env = roblox_vm.get_mut().unwrap().get_main_state().get_lua().globals();
                roblox_vm.get_mut().unwrap()
                    .get_main_state()
                    .compile_jit("meow?", r#"
                        print("Hey there!", _VERSION, "running in Godot!")
                        warn("uwu")
                        print(Vector3int16.new(32768, 10, -11))
                        print(Instance)
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
