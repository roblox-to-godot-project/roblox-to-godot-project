#![allow(dead_code)]

#![feature(trait_upcasting)]
#![feature(ptr_metadata)]
#![feature(arbitrary_self_types)]
#![feature(negative_impls)]
#![feature(variant_count)]

#![allow(internal_features)]
#![feature(core_intrinsics)]

#[rustversion::not(nightly)]
compile_error!("This crate can only be built with nightly rust due to the use of unstable features.");

pub mod core;
pub mod instance;
pub mod userdata;
mod godot_vm_bindings;

use core::verify_gdext_api_compat;

pub use godot_vm_bindings::RobloxVMNode;

use godot::prelude::*;
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

struct RobloxToGodotProjectExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RobloxToGodotProjectExtension {
    fn min_level() -> InitLevel {
        InitLevel::Scene
    }

    fn on_level_init(level: InitLevel) {
        
        match level {
            InitLevel::Scene => {
                verify_gdext_api_compat();
                godot_print!("Roblox To Godot Project v{} (Rust runtime v{}) by {}\n", env!("CARGO_PKG_VERSION"), RUST_VERSION, {
                    let authors: &'static str = env!("CARGO_PKG_AUTHORS");
                    authors.replace(":", ", ")
                });
                /*
                let mut roblox_vm = RobloxVM::new(None);
                let env = roblox_vm.get_mut().get_main_state().create_env_from_global().unwrap();
                roblox_vm.get_mut()
                    .get_main_state()
                    .compile_jit("test.lua", include_str!("test.lua"), env).unwrap()
                    .call::<()>(()).unwrap();
                */
                
            }
            _ => ()
        }
    }
}

// #[roblox_to_godot_project_derive::instance(hierarchy=[ServiceProvider], no_clone, parent_locked)]
// struct TestInstance {
//     #[property(name = "meow", readonly)]
//     meow: u32,
//     a_field: String,
//     #[method(name = "regret")]
//     fn why_did_i_do_this() {

//     },
//     #[property(name = "owo", readonly)]
//     another_field: String,
// }