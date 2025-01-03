use std::{sync::RwLock, thread::panicking};
use std::marker::PhantomPinned;

use godot::global::godot_print_rich;
use godot::{builtin::Variant, global::{godot_print, print_rich, printt}, meta::ToGodot};
use mlua::prelude::*;

use super::state::LuauState;
use super::InstanceReplicationTable;

pub struct RobloxVM {
    main_state: LuauState,
    states: Vec<RwLock<LuauState>>,
    instances: InstanceReplicationTable,
    _pin: PhantomPinned
}

pub(crate) fn args_to_variant(args: LuaMultiValue) -> Box<[Variant]> {
    args
        .into_iter()
        .map(|x| {
            x.to_string().unwrap_or("<unknown>".into()).as_str().to_variant()
        })
        .collect()
}
pub(crate) fn args_to_string(args: LuaMultiValue, delimiter: &str) -> String {
    let mut iter = args
        .into_iter()
        .map(|x|
            String::from(x.to_string().unwrap_or("<unknown>".into()))
        );
    let first = iter.next().unwrap_or(String::default());
    iter.fold(first, |mut a, b| {
            a.push_str(delimiter);
            a.push_str(b.as_str());
            a
        })
}

impl RobloxVM {
    pub fn new() -> Box<RwLock<RobloxVM>> {
        unsafe {
            let mut vm = Box::new(RwLock::new(RobloxVM {
                main_state: LuauState::new_uninit(),
                states: Vec::new(),
                instances: InstanceReplicationTable::default(),
                _pin: PhantomPinned::default()
            }));
            let vm_ptr = &raw mut *vm;

            vm.get_mut().unwrap_unchecked().main_state.init(vm_ptr);
            godot_print!("RobloxVM instance created.");
            vm
        }
    }
    pub fn log_message(&self, args: LuaMultiValue) {
        let v = args_to_variant(args);
        printt(&v);
    }
    pub fn log_info(&self, args: LuaMultiValue) {
        let mut string = args_to_string(args, "\t");
        string = "[color=blue]".to_owned() + &string;
        string = string + "[/color]";
        let v: [Variant; 1] = [string.to_variant()];
        print_rich(&v)
    }
    pub fn log_warn(&self, args: LuaMultiValue) {
        let mut string = args_to_string(args, "\t");
        string = "[color=yellow]".to_owned() + &string;
        string = string + "[/color]";
        let v: [Variant; 1] = [string.to_variant()];
        print_rich(&v)
    }
    pub fn log_err(&self, args: LuaMultiValue) {
        let mut string = args_to_string(args, "\t");
        string = "[color=red]".to_owned() + &string;
        string = string + "[/color]";
        let v: [Variant; 1] = [string.to_variant()];
        print_rich(&v)
    }
    pub fn get_main_state(&mut self) -> &mut LuauState {
        &mut self.main_state
    }
}

impl Drop for RobloxVM {
    fn drop(&mut self) {
        if panicking() {
            godot_print_rich!("[color=red][b]ERROR: RobloxVM:[/b] Abnormal exit (panicking() == true)[/color]\n[color=gray]\tat RobloxVM::drop() ({}:{})[/color]", file!(), line!());
        }
        self.states.clear();
        godot_print!("RobloxVM instance destroyed.");
    }
}
