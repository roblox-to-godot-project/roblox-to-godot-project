use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::thread::panicking;
use std::marker::PhantomPinned;

use godot::global::godot_print_rich;
use godot::{builtin::Variant, global::{godot_print, print_rich, printt}, meta::ToGodot};
use mlua::prelude::*;

use crate::core::scheduler::GlobalTaskScheduler;

use super::state::LuauState;
use super::{FastFlag, FastFlagValue, FastFlags, InstanceReplicationTable, InstanceTagCollectionTable, RwLock, Watchdog};

pub struct RobloxVM {
    main_state: RwLock<LuauState>,
    states: Vec<Option<RwLock<LuauState>>>,
    instances: InstanceReplicationTable,
    instances_tag_collection: InstanceTagCollectionTable,
    flags: MaybeUninit<FastFlags>,

    states_locks: HashMap<*mut LuauState, *const RwLock<LuauState>>,
    
    hard_wd: Watchdog,
    soft_wd: Watchdog,

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
    pub fn new(flags_table: Option<Vec<(FastFlag, FastFlagValue)>>) -> Box<RwLock<RobloxVM>> {
        unsafe {
            let mut vm = Box::new(RwLock::new(RobloxVM {
                main_state: RwLock::new(LuauState::new_uninit()),
                states: Vec::new(),
                states_locks: HashMap::new(),
                instances: InstanceReplicationTable::default(),
                instances_tag_collection: InstanceTagCollectionTable::default(),
                hard_wd: Watchdog::new_timeout(10.0),
                soft_wd: Watchdog::new_timeout(1.0/60.0),
                _pin: PhantomPinned::default(),
                flags: MaybeUninit::uninit()
            }));
            let vm_ptr = &raw mut *vm;
            vm.get_mut().flags.write(FastFlags::new(vm_ptr));
            if let Some(table) = flags_table {
                vm.get_mut().flags.assume_init_mut()
                    .initialize_with_table(table);
            }
            let main_state_ptr = vm.get_mut().main_state.access();
            let main_state_lock_ptr = &raw const vm.get_mut().main_state;
            vm.get_mut().states_locks.insert(main_state_ptr, main_state_lock_ptr);

            vm.get_mut().main_state.get_mut().init(vm_ptr, Box::new(GlobalTaskScheduler::new()));
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
        unsafe { &mut *self.main_state.access() }
    }
    pub(super) fn get_state_with_rwlock(&self, ptr: *mut LuauState) -> Option<*const RwLock<LuauState>> {
        self.states_locks.get(&ptr).map(|x| *x)
    }
    unsafe fn watchdog_trip_state(state: *mut LuauState) {
        state.as_mut().unwrap_unchecked().get_lua().set_interrupt(
            |_| Err(LuaError::RuntimeError("script exhausted maximum execution time".into()))
        );
    }
    fn watchdog_reset_state(state: &mut LuauState) {
        state.get_lua().remove_interrupt();
    }
    pub fn watchdog_trip(&self) {
        self.hard_wd.trip();
        // SAFETY: Luau permits setting interrupt from other threads.
        unsafe { 
            Self::watchdog_trip_state(self.main_state.access());
            for i in self.states.iter() {
                i.as_ref().map(|x| {
                    Self::watchdog_trip_state(x.access());
                });
            }
        }
    }
    pub fn watchdog_reset(&mut self) {
        if self.hard_wd.check() {
            Self::watchdog_reset_state(unsafe { self.main_state.access().as_mut().unwrap_unchecked() });
            for i in self.states.iter() {
                i.as_ref().map(|x| {
                    Self::watchdog_reset_state(x.write().unwrap().borrow_mut());
                });
            }
        }
        self.hard_wd.reset();
        self.soft_wd.reset();
    }
    pub(crate) fn watchdog_check(&self) -> bool {
        if self.hard_wd.check() {
            self.watchdog_trip();
        }
        self.soft_wd.check()
    }
    #[inline(always)]
    pub(crate) fn get_instance_tag_table(&self) -> &InstanceTagCollectionTable {
        &self.instances_tag_collection
    }
    #[inline(always)]
    pub(crate) const fn flags(&self) -> &FastFlags {
        unsafe { self.flags.assume_init_ref() }
    }
}

impl Drop for RobloxVM {
    fn drop(&mut self) {
        if panicking() {
            godot_print_rich!("[color=red][b]ERROR: RobloxVM:[/b] Abnormal exit (panicking() == true)[/color]\n[color=gray]   at RobloxVM::drop() ({}:{})[/color]", file!(), line!());
        }
        self.states.clear();
        unsafe { self.flags.assume_init_drop() };
        godot_print!("RobloxVM instance destroyed.");
    }
}