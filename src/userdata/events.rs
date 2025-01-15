use std::{cell::RefCell, collections::HashMap, future::Future, mem::take, pin::Pin, rc::Rc, task::{Context, Poll}};

use mlua::prelude::*;
use super::from_lua_clone_impl;
use crate::core::{get_state_with_rwlock, get_task_scheduler_from_lua, LuauState, ParallelDispatch, RwLock, Trc, TrcReadLock, TrcWriteLock, Weak};
pub type ManagedRBXScriptSignal = Trc<RBXScriptSignal>;

#[derive(Debug, Clone)]
pub struct RBXScriptConnection {
    id: usize,
    signal: ManagedRBXScriptSignal
}
#[derive(Debug, Clone)]
struct SignalCallback {
    func: LuaFunction,
    state: *const RwLock<LuauState>,
    once: bool,
    parallel: ParallelDispatch
}
#[derive(Debug)]
pub struct RBXScriptSignal {
    callbacks: HashMap<usize, SignalCallback>,
    this_ptr: Option<Weak<RBXScriptSignal>>,
    id: usize
}
struct InnerRBXScriptSignalFuture {
    event: ManagedRBXScriptSignal,
    lua: Lua,
    resolved: bool,
    waiting: bool,
    values: LuaMultiValue
}
pub struct RBXScriptSignalFuture {
    future: Rc<RefCell<InnerRBXScriptSignalFuture>>
}

impl RBXScriptSignal {
    pub fn new() -> Trc<RBXScriptSignal> {
        Trc::new_cyclic(|x| RBXScriptSignal { callbacks: HashMap::default(), this_ptr: Some(x.clone()), id: 0 })
    }
    pub fn connect(&mut self, lua: &Lua, func: LuaFunction, parallel: ParallelDispatch) -> LuaResult<RBXScriptConnection> {
        let id = self.id;
        self.id += 1;
        self.callbacks.insert(id, SignalCallback {
            func,
            state: get_state_with_rwlock(lua),
            once: false,
            parallel: parallel
        });
        Ok(RBXScriptConnection {
            id,
            signal: self.this_ptr.as_ref().unwrap().upgrade().unwrap()
        })
    }
    #[inline]
    pub fn connect_parallel(&mut self, lua: &Lua, func: LuaFunction) -> LuaResult<RBXScriptConnection> {
        self.connect(lua, func, ParallelDispatch::Desynchronized)
    }
    pub fn once(&mut self, lua: &Lua, func: LuaFunction, parallel: ParallelDispatch) -> LuaResult<RBXScriptConnection> {
        let id = self.id;
        self.id += 1;
        self.callbacks.insert(id, SignalCallback {
            func,
            state: get_state_with_rwlock(lua),
            once: true,
            parallel: parallel
        });
        Ok(RBXScriptConnection {
            id,
            signal: self.this_ptr.as_ref().unwrap().upgrade().unwrap()
        })
    }
    pub fn fire(mut self: TrcWriteLock<'_, RBXScriptSignal>, lua: &Lua, args: impl IntoLuaMulti) -> LuaResult<()> {
        let args = args.into_lua_multi(lua)?;
        let mut to_remove = Vec::new();
        let task = get_task_scheduler_from_lua(unsafe {(lua as *const Lua).as_ref().unwrap_unchecked()});
        let callbacks_clone = self.callbacks.clone();
        let release = self.guard_release();
        for (id, callback) in callbacks_clone {
            //let _ = task.defer_func(lua, callback.func, args.clone(), callback.parallel);
            let _ = task.spawn_func(lua, callback.func, args.clone());
            if callback.once {
                to_remove.push(id);
            }
        }
        drop(release);
        if !to_remove.is_empty() {
            for i in to_remove {
                self.callbacks.remove(&i);
            }
        }
        Ok(())
    }
    pub fn wait<'a>(self: TrcReadLock<'_, Self>, lua: &'a Lua) -> RBXScriptSignalFuture {
        RBXScriptSignalFuture {
            future: Rc::new(RefCell::new(InnerRBXScriptSignalFuture {
                event: self.this_ptr.as_ref().unwrap().upgrade().unwrap(),
                lua: lua.clone(),
                resolved: false,
                waiting: false,
                values: LuaMultiValue::new()
            }))
        }
    }
}
impl RBXScriptConnection {
    pub fn is_connected(&self) -> bool {
        self.signal.read().callbacks.contains_key(&self.id)
    }
    pub fn disconnect(&self) {
        let mut lock = self.signal.write();
        lock.callbacks.remove(&self.id);
    }
}

impl Future for RBXScriptSignalFuture {
    type Output = LuaResult<LuaMultiValue>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let immut_borrow = self.future.borrow();
        if immut_borrow.resolved {
            drop(immut_borrow);
            let mut borrow = self.future.borrow_mut();
            borrow.resolved = false;
            borrow.waiting = false;
            let values = take(&mut borrow.values);
            Poll::Ready(Ok(values))
        } else {
            if !immut_borrow.waiting {
                let waker = cx.waker().clone();
                let clone = self.future.clone();
                let func = immut_borrow.lua.create_function_mut(move |_, mv: LuaMultiValue| {
                    let mut borrow = clone.borrow_mut();
                    borrow.resolved = true;
                    borrow.waiting = false;
                    borrow.values = mv;
                    waker.clone().wake();
                    Ok(())
                })?;
                immut_borrow.event.write().once(&immut_borrow.lua, func, ParallelDispatch::Default)?;
            }
            Poll::Pending
        }
    }
}
impl LuaUserData for ManagedRBXScriptSignal {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("Connect", |lua, this, func: LuaFunction| {
            this.write().connect(lua, func, ParallelDispatch::Synchronized)
        });
        methods.add_method_mut("ConnectParallel", |lua, this, func: LuaFunction| {
            this.write().connect_parallel(lua, func)
        });
        methods.add_method_mut("Once", |lua, this, func: LuaFunction| {
            this.write().once(lua, func, ParallelDispatch::Synchronized)
        });
        methods.add_async_method_mut("Wait", async |lua, this, ()| {
            this.read().wait(&lua).await
        });
    }
}
impl LuaUserData for RBXScriptConnection {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("Disconnect", 
        |_, this, ()|
            Ok(this.disconnect())
        );
    }
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Connected", 
            |_, this|
                Ok(this.is_connected())
        );
    }
}

from_lua_clone_impl!(RBXScriptConnection);
from_lua_clone_impl!(ManagedRBXScriptSignal);