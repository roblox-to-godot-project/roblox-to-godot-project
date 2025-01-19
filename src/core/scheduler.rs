use std::mem::take;

use mlua::{ffi::{self, lua_resume, lua_settop, lua_tothread}, prelude::*};
use super::{get_state, RobloxVM, RwLockWriteGuard};

#[derive(Debug)]
pub struct TaskScheduler {
    defer_threads: [Vec<(LuaThread, u32)>; 2],
    delay_threads: [Vec<(LuaThread, u32, f64)>; 2],
    parallel_dispatch: bool
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ParallelDispatch {
    Desynchronized,
    Default,
    Synchronized
}

impl Default for ParallelDispatch {
    fn default() -> Self {
        Self::Default
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ITaskScheduler {
    fn get_task_scheduler(&self) -> &TaskScheduler;
    fn get_task_scheduler_mut(&mut self) -> &mut TaskScheduler;
}
impl TaskScheduler {
    pub fn as_dyn(&self) -> &dyn ITaskScheduler {
        unsafe { & *((&raw const *self) as *const dyn ITaskScheduler) }
    }
    pub fn as_dyn_mut(&mut self) -> &mut dyn ITaskScheduler {
        unsafe { &mut *((&raw mut *self) as *mut dyn ITaskScheduler) }
    }
    pub(crate) const fn new() -> TaskScheduler {
        TaskScheduler {
            defer_threads: [Vec::new(),Vec::new()],
            delay_threads: [Vec::new(),Vec::new()],
            parallel_dispatch: false
        }
    }
}
impl ITaskScheduler for TaskScheduler {
    fn get_task_scheduler(&self) -> &TaskScheduler { self }
    fn get_task_scheduler_mut(&mut self) -> &mut TaskScheduler { self }
}
impl dyn ITaskScheduler {
    #[inline(always)]
    fn clock() -> f64 {
        unsafe { ffi::lua_clock() }
    }
    #[inline(always)]
    fn dispatch_to_table(&self, parallel: ParallelDispatch) -> usize {
        (match parallel {
            ParallelDispatch::Synchronized => false,
            ParallelDispatch::Default => self.get_task_scheduler().parallel_dispatch,
            ParallelDispatch::Desynchronized => true,
        }) as usize
    }
    pub fn spawn_func(&self, lua: &Lua, func: LuaFunction, args: impl IntoLuaMulti) -> LuaResult<LuaThread> {
        let thread = lua.create_thread(func)?;
        let _: LuaResult<()> = thread.resume(args);
        Ok(thread)
    }
    pub fn spawn_thread(&self, thr: LuaThread) -> LuaResult<LuaThread> {
        let _ = thr.resume::<()>(());
        Ok(thr)
    }
    pub fn defer_high_priority<F, A, R>(&mut self, lua: &Lua, args: impl IntoLuaMulti, parallel: ParallelDispatch, f: F) -> LuaResult<LuaThread> 
    where
        F: FnMut(&Lua, A) -> LuaResult<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti
    {
        let parallel = self.dispatch_to_table(parallel);
        let func = lua.create_function_mut(f)?;
        let thread = lua.create_thread(func)?;
        let mut args_count: i32 = 0;
        unsafe { 
            lua.exec_raw::<()>((thread.clone(), args),|lua_raw: *mut ffi::lua_State| {
                args_count = ffi::lua_gettop(lua_raw) - 1;
                let thread = ffi::lua_tothread(lua_raw, 1);
                
                ffi::lua_xmove(lua_raw, thread, args_count);
                ffi::lua_settop(lua_raw, 0); // clear stack
            })?;
        }
        self.get_task_scheduler_mut().defer_threads[parallel].insert(0, (thread.clone(), args_count as u32));
        Ok(thread)
    }
    pub fn defer_func(&mut self, lua: &Lua, func: LuaFunction, args: impl IntoLuaMulti, parallel: ParallelDispatch) -> LuaResult<LuaThread> {
        let parallel = self.dispatch_to_table(parallel);
        let thread = lua.create_thread(func)?;
        let mut args_count: i32 = 0;
        unsafe { 
            lua.exec_raw::<()>((thread.clone(), args),|lua_raw: *mut ffi::lua_State| {
                args_count = ffi::lua_gettop(lua_raw) - 1;
                let thread = ffi::lua_tothread(lua_raw, 1);
                
                ffi::lua_xmove(lua_raw, thread, args_count);
                ffi::lua_settop(lua_raw, 0); // clear stack
            })?;
        }
        self.get_task_scheduler_mut().defer_threads[parallel].push((thread.clone(), args_count as u32));
        Ok(thread)
    }
    pub fn defer_thread(&mut self, thread: LuaThread, parallel: ParallelDispatch) -> LuaResult<LuaThread> {
        let parallel = self.dispatch_to_table(parallel);
        self.get_task_scheduler_mut().defer_threads[parallel].push((thread.clone(), 0));
        Ok(thread)
    }
    pub fn delay_func(&mut self, lua: &Lua, func: LuaFunction, args: impl IntoLuaMulti, parallel: ParallelDispatch, seconds: f64) -> LuaResult<LuaThread> {
        let parallel = self.dispatch_to_table(parallel);
        let thread = lua.create_thread(func)?;
        let mut args_count: i32 = 0;
        unsafe { 
            lua.exec_raw::<()>((thread.clone(), args),|lua_raw: *mut ffi::lua_State| {
                args_count = ffi::lua_gettop(lua_raw) - 1;
                let thread = ffi::lua_tothread(lua_raw, 1);
                
                ffi::lua_xmove(lua_raw, thread, args_count);
                ffi::lua_settop(lua_raw, 0); // clear stack
            })?;
        }
        self.get_task_scheduler_mut().delay_threads[parallel].push((thread.clone(), args_count as u32, Self::clock() + seconds));
        Ok(thread)
    }
    pub fn delay_thread(&mut self, thread: LuaThread, parallel: ParallelDispatch, seconds: f64) -> LuaResult<LuaThread> {
        let parallel = self.dispatch_to_table(parallel);
        self.get_task_scheduler_mut().delay_threads[parallel].push((thread.clone(), 0, Self::clock() + seconds));
        Ok(thread)
    }
    pub fn defer_single_cycle<'a>(&mut self, lua: &'a Lua, parallel: bool) -> LuaResult<bool> {
        self.get_task_scheduler_mut().parallel_dispatch = parallel;
        let threads = take(&mut self.get_task_scheduler_mut().defer_threads[parallel as usize]);
        for (thread, args) in threads { unsafe {
            if thread.status() == LuaThreadStatus::Resumable {
                lua.exec_raw::<()>((thread,), |lua_raw: *mut ffi::lua_State| {
                    let thread = lua_tothread(lua_raw, 1);
                    let mut _nres = 0;
                    lua_resume(thread, lua_raw, args as i32, &raw mut _nres);
                    lua_settop(lua_raw, 0);
                })?;
            }
        }}
        Ok(!self.get_task_scheduler_mut().defer_threads[parallel as usize].is_empty())
    }
    pub fn delay_single_cycle<'a>(&mut self, lua: &'a Lua, parallel: bool) -> LuaResult<bool> {
        self.get_task_scheduler_mut().parallel_dispatch = parallel;
        let threads = take(&mut self.get_task_scheduler_mut().delay_threads[parallel as usize]);
        let new_threads  = &mut self.get_task_scheduler_mut().delay_threads[parallel as usize];
        new_threads.reserve(threads.len());
        for (thread, args, time ) in threads { unsafe {
            if Self::clock() >= time {
                if thread.status() == LuaThreadStatus::Resumable {
                    lua.exec_raw::<()>((thread,), |lua_raw: *mut ffi::lua_State| {
                        let thread = lua_tothread(lua_raw, 1);
                        let mut _nres = 0;
                        lua_resume(thread, lua_raw, args as i32, &raw mut _nres);
                        lua_settop(lua_raw, 0);
                    })?;
                }
            } else {
                new_threads.push((thread, args, time));
            }
        }}
        Ok(!new_threads.is_empty())
    }
    #[inline]
    fn watchdog_check(&self, lua: &Lua) -> bool {
        unsafe { get_state(lua).watchdog_check() }
    }
    pub fn defer_cycle<'a>(&mut self, lua: &'a Lua, parallel: bool) -> LuaResult<()> {
        while self.defer_single_cycle(lua, parallel)? && !self.watchdog_check(lua) {}
        Ok(())
    }
    pub fn delay_cycle(&mut self, lua: &Lua, parallel: bool) -> LuaResult<()> {
        while self.delay_single_cycle(lua, parallel)? && !self.watchdog_check(lua) {}
        Ok(())
    }
    pub fn is_desynchronized(&self) -> bool {
        self.get_task_scheduler().parallel_dispatch
    }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct GlobalTaskScheduler {
    task: TaskScheduler
}
impl GlobalTaskScheduler {
    pub fn as_dyn(&self) -> &dyn ITaskScheduler {
        unsafe { & *((&raw const self.task) as *const dyn ITaskScheduler) }
    }
    pub fn as_dyn_mut(&mut self) -> &mut dyn ITaskScheduler {
        unsafe { &mut *((&raw mut self.task) as *mut dyn ITaskScheduler) }
    }
    pub fn frame_step(mut vm: RwLockWriteGuard<RobloxVM>, _: f64) -> LuaResult<()> {
        
        // SAFETY: This function avoids the borrow checker since the main state outlives global task scheduler.
        let main_state_ptr = &raw mut *vm.get_main_state();
        let main_state = unsafe {main_state_ptr.as_mut().unwrap_unchecked()};

        let lua = main_state.get_lua().clone();
        let task = main_state.get_task_scheduler_mut();
        task.get_task_scheduler_mut().parallel_dispatch = false;
        vm.watchdog_reset();
        task.defer_cycle(&lua, false)?;
        task.delay_cycle(&lua, false)?;
        // todo! run service events
        drop(vm);
        Ok(())
    }
    pub fn new() -> GlobalTaskScheduler {
        GlobalTaskScheduler {
            task: TaskScheduler::new()
        }
    }
}
impl ITaskScheduler for GlobalTaskScheduler {
    fn get_task_scheduler(&self) -> &TaskScheduler {
        &self.task
    }

    fn get_task_scheduler_mut(&mut self) -> &mut TaskScheduler {
        &mut self.task
    }
}

pub fn get_task_scheduler_from_lua<'a, 'b>(lua: &'a Lua) -> &'b mut dyn ITaskScheduler {
    // SAFETY: Lua and the task scheduler must have same lifetime.
    get_state(unsafe {(lua as *const Lua).as_ref().unwrap_unchecked()}).get_task_scheduler_mut()
}