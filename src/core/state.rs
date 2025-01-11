use std::mem::MaybeUninit;
use std::{collections::HashMap, ffi::c_void, mem::transmute, ptr::addr_of_mut};
use std::ptr::null_mut;

use godot::global::godot_print;
use mlua::{prelude::*, ChunkMode, Compiler};
use super::scheduler::ITaskScheduler;
use super::{RwLock, RwLockReadGuard, RwLockWriteGuard, TaskScheduler};
use super::{security::ThreadIdentityType, vm::RobloxVM};
use crate::userdata::register_userdata_singletons;

pub mod registry_keys {
    pub const VM_REGISTRYKEY: &'static str = "__vm__";
    pub const STATE_REGISTRYKEY: &'static str = "__state__";
}
#[derive(Clone, PartialEq, Eq)]
pub struct ThreadIdentity {
    pub security_identity: ThreadIdentityType,
    pub script: Option<()> // TODO: Add the type
}

pub struct LuauState {
    vm: *mut RwLock<RobloxVM>,
    lua: Lua,
    threads: HashMap<*const c_void, ThreadIdentity>,
    task: MaybeUninit<Box<dyn ITaskScheduler>>
}
impl LuauState {
    fn get_vm_from_lua(lua: &Lua) -> &RwLock<RobloxVM> {
        unsafe {
            let ptr: *mut RwLock<RobloxVM> = transmute(
                lua.named_registry_value::<LuaLightUserData>(registry_keys::VM_REGISTRYKEY)
                .unwrap_unchecked().0
            );
            debug_assert!(!ptr.is_null());
            ptr.as_mut().unwrap_unchecked()
        }
    }
    fn thread_event_callback(lua: &Lua, event: LuaThreadEventInfo) -> Result<(), LuaError> {
        let state;
        unsafe {
            let reg: LuaLightUserData = lua.named_registry_value(registry_keys::STATE_REGISTRYKEY)?;
            state = reg.0.cast::<LuauState>().as_mut().unwrap();
        }
        match event {
            LuaThreadEventInfo::Created(parent) => {
                godot_print!("LuaThreadEventInfo::Created(child: thread: 0x{:x},parent: thread: 0x{:x})",lua.current_thread().to_pointer() as isize,parent.to_pointer() as isize);
                let iden = state.threads.get(&parent.to_pointer());
                if iden.is_some() {
                    state.threads.insert(lua.current_thread().to_pointer(), iden.unwrap().clone());
                }
                Ok(())
            }
            LuaThreadEventInfo::Destroyed(thread_ptr) => {
                godot_print!("LuaThreadEventInfo::Destroyed(thread: 0x{:x})",thread_ptr as isize);
                state.threads.remove(&thread_ptr);
                Ok(())
            }
        }
    }
    unsafe fn _init(&mut self) {
        self.lua.set_named_registry_value(registry_keys::VM_REGISTRYKEY, LuaLightUserData(self.vm.cast())).unwrap();
        let state_userdata = LuaLightUserData(addr_of_mut!(*self).cast());
        self.lua.set_named_registry_value(registry_keys::STATE_REGISTRYKEY, state_userdata).unwrap();
        self.lua.globals().raw_set("print", self.lua.create_function(|lua, args: LuaMultiValue| {
            let vm = Self::get_vm_from_lua(lua);
            vm.read().unwrap().log_message(args);
            Ok(())
        }).unwrap()).unwrap();
        self.lua.globals().raw_set("warn", self.lua.create_function(|lua, args: LuaMultiValue| {
            let vm = Self::get_vm_from_lua(lua);
            vm.read().unwrap().log_warn(args);
            Ok(())
        }).unwrap()).unwrap();
        self.lua.sandbox(true).unwrap();
        self.lua.enable_jit(false);
        register_userdata_singletons(&mut self.lua).unwrap();
        self.lua.globals().set_readonly(self.get_vm().are_globals_readonly());
        self.lua.set_thread_event_callback(Self::thread_event_callback);
    }
    pub(super) unsafe fn init(&mut self, ptr: *mut RwLock<RobloxVM>, task: Box<dyn ITaskScheduler>) {
        self.vm = ptr;
        self.task = MaybeUninit::new(task);
        self._init();
    }
    #[doc(hidden)]
    pub(super) fn new(ptr: *mut RwLock<RobloxVM>) -> LuauState {
        let mut state = LuauState {
            vm: ptr,
            lua: Lua::new(),
            threads: HashMap::default(),
            task: MaybeUninit::new(Box::new(TaskScheduler::new()))
        };
        unsafe {state._init();}
        state
    }
    pub(super) unsafe fn new_uninit() -> LuauState {
        LuauState {
            vm: null_mut(),
            lua: Lua::new(),
            threads: HashMap::default(),
            task: MaybeUninit::uninit()
        }
    }
    pub fn get_vm(&self) -> RwLockReadGuard<RobloxVM> {
        unsafe {
            self.vm.as_ref().unwrap_unchecked().read().unwrap()
        }
    }
    pub fn get_vm_mut(&self) -> RwLockWriteGuard<RobloxVM> {
        unsafe {
            self.vm.as_ref().unwrap_unchecked().write().unwrap()
        }
    }

    pub(super) unsafe fn watchdog_check(&self) -> bool {
        self.vm.as_ref().unwrap_unchecked().access().as_ref().unwrap_unchecked().watchdog_check()
    }

    // Mutable borrow is forced here to prevent modifying the lua state with a read-only borrow.
    pub fn get_lua(&mut self) -> &mut Lua {
        &mut self.lua
    }
    pub fn get_userdata_types() -> &'static [&'static str] {
        /*&[
            "Axes","BrickColor","CatalogSearchParams","CFrame","ColorSequence","ColorSequenceKeypoint","Content","DateTime",
            "DockWidgetPluginGuiInfo","Enum","EnumItem","Enums","Faces","FloatCurveKey","Font","Instance","NumberRange",
            "NumberSequence","NumberSequenceKeypoint","OverlapParams","Path2DControlPoint","PathWaypoint","PhysicalProperties",
            "Random","Ray","RaycastParams","RaycastResult","RBXScriptConnection","RBXScriptSignal","Rect","Region3","Region3int16",
            "RotationCurveKey","Secret","SharedTable","TweenInfo","UDim","UDim2","Vector2","Vector2int16","Vector3","Vector3int16"
        ]*/ // sadly doesnt work
        &["Instance"] // good enough
    }
    fn get_debug_compiler() -> Compiler {
        Compiler::new()
            .set_debug_level(2)
            .set_optimization_level(1)
            .set_userdata_types(Self::get_userdata_types().into_iter().map(|x| String::from(*x)).collect())
    }
    fn get_release_compiler() -> Compiler {
        Compiler::new()
            .set_debug_level(1)
            .set_optimization_level(2)
            .set_userdata_types(Self::get_userdata_types().into_iter().map(|x| String::from(*x)).collect())
            .set_type_info_level(1)
    }
    pub fn compile_jit(&mut self, chunk_name: &str, chunk: &str, env: LuaTable) -> LuaResult<LuaFunction> {
        let v = Self::get_release_compiler()
            .compile(chunk)?;
        self.lua.enable_jit(true);
        let f = self.lua
            .load(v)
            .set_name(chunk_name)
            .set_mode(ChunkMode::Binary)
            .set_environment(env)
            .into_function();
        self.lua.enable_jit(false);
        f
    }
    pub fn compile_release(&mut self, chunk_name: &str, chunk: &str, env: LuaTable) -> LuaResult<LuaFunction> {
        let v = Self::get_release_compiler()
            .compile(chunk)?;
        self.lua.load(v)
            .set_name(chunk_name)
            .set_mode(ChunkMode::Binary)
            .set_environment(env)
            .into_function()
    }
    pub fn compile_debug(&mut self, chunk_name: &str, chunk: &str, env: LuaTable) -> LuaResult<LuaFunction> {
        let v = Self::get_debug_compiler()
            .compile(chunk)?;
        self.lua.load(v)
            .set_name(chunk_name)
            .set_mode(ChunkMode::Binary)
            .set_environment(env)
            .into_function()
    }
    pub fn create_env_from_global(&mut self) -> LuaResult<LuaTable> {
        let lua = self.get_lua();
        let metatable = lua.create_table()?;
        metatable.raw_set("__metatable", "env")?;
        metatable.raw_set("__index", lua.globals())?;

        let table = lua.create_table()?;
        table.set_metatable(Some(metatable));

        Ok(table)
    }
    pub fn get_task_scheduler_mut(&mut self) -> &mut dyn ITaskScheduler {
        // SAFETY: The state must be initialized
        unsafe { &mut **self.task.assume_init_mut() }
    }
    pub fn get_task_scheduler(&self) -> &dyn ITaskScheduler {
        // SAFETY: The state must be initialized
        unsafe { &**self.task.assume_init_ref() }
    }
}

pub fn get_current_identity(l: &Lua) -> Option<&mut ThreadIdentity> {
    let state;
    unsafe {
        let reg: Option<LuaLightUserData> = l.named_registry_value(registry_keys::STATE_REGISTRYKEY).ok();
        if reg.is_none() {
            return None;
        }
        state = reg.unwrap_unchecked().0.cast::<LuauState>().as_mut().unwrap();
    }
    state.threads.get_mut(&l.current_thread().to_pointer().cast())
}
// SAFETY: This function is safe because lua instances can be obtained only if its inside of it or if it already holds a lock.
pub fn get_state(l: &Lua) -> &mut LuauState {
    let state;
    unsafe {
        let reg = l.named_registry_value::<LuaLightUserData>(registry_keys::STATE_REGISTRYKEY);
        state = reg.unwrap().0.cast::<LuauState>().as_mut().unwrap();
    }
    state
}
pub fn get_state_with_rwlock(l: &Lua) -> &RwLock<LuauState> {
    let state = get_state(l);
    let ptr = &raw mut *state;
    let vm = state.get_vm();
    vm.get_state_with_rwlock(ptr).map(|x| unsafe {&*x}).unwrap()
}