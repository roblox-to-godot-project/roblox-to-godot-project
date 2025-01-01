use mlua::prelude::*;

use crate::instance::{ManagedInstance};

use super::LuaSingleton;

impl LuaUserData for ManagedInstance {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__index", |lua, this, field: String| {
            let instance_read = this.read().expect("instance poisoned");
            instance_read.lua_get(lua, field)
        });
        methods.add_meta_method("__newindex", |lua, this, (field, val): (String, LuaValue)| {
            let mut instance_write = this.write().expect("instance poisoned");
            instance_write.lua_set(lua, field, val)
        });
        methods.add_meta_method("__tostring", |_, this: &ManagedInstance, ()| {
            let instance_read = this.read().expect("instance poisoned");
            Ok(format!("{} {}: {}", instance_read.get_class_name(), instance_read.get_name(), instance_read.get_uniqueid()))
        });
    }
}

impl LuaSingleton for ManagedInstance {
    fn register_singleton(lua: &Lua) -> LuaResult<()> {
        let table = lua.create_table()?;
        table.raw_set("new", lua.create_function(|_, (class_name,): (String,)| {
            match class_name.as_str() {
                _ => Err::<LuaValue, LuaError>(LuaError::RuntimeError(format!("invalid class name \"{}\"", class_name)))
            }
        })?)?;
        lua.globals().raw_set("Instance", table)?;
        Ok(())
    }
}