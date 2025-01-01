use std::{collections::HashMap, sync::RwLock};

use mlua::prelude::*;
//use crate::instance::WeakManagedInstance;
use crate::core::{LuauState, Trc, Weak};
pub type ManagedRBXScriptSignal = Trc<RBXScriptSignal>;

pub struct RBXScriptConnection {

}
#[derive(Default)]
pub struct RBXScriptSignal {
    callbacks: HashMap<usize, (LuaFunction, *const RwLock<LuauState>)>
}
impl RBXScriptSignal {}
impl LuaUserData for ManagedRBXScriptSignal {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("Connect", |lua, this, mult: LuaMultiValue| {
            todo!();
            Ok(())
        });
    }
}