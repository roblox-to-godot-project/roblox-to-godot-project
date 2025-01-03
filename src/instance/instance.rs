use std::collections::HashMap;
use std::fmt::Debug;
use std::ptr::NonNull;

use crate::core::alloc::Allocator;
use crate::core::lua_macros::lua_getter;
use crate::core::{ITrc, ITrcHead, IWeak};
use crate::userdata::{ManagedRBXScriptSignal, RBXScriptSignal};
use mlua::prelude::*;

use super::IObject;

pub type ManagedInstance = ITrc<dyn IInstance>;
pub type WeakManagedInstance = IWeak<dyn IInstance>;
pub type EventsTable = HashMap<String, ManagedRBXScriptSignal>;

pub trait IInstanceComponent: Sized {
    unsafe fn weak_to_strong_instance(ptr: WeakManagedInstance) -> ManagedInstance {
        ptr.upgrade().unwrap_unchecked()
    }
    fn lua_get(&self, ptr: WeakManagedInstance, lua: &Lua, key: &String) -> Option<LuaResult<LuaValue>>;
    fn lua_set(&mut self, ptr: WeakManagedInstance, lua: &Lua, key: &String, value: &LuaValue) -> Option<LuaResult<()>>;
    fn clone(&self, new_ptr: WeakManagedInstance) -> LuaResult<Self>;
    fn new(ptr: WeakManagedInstance, class_name: &'static str) -> Self;
}

pub trait IInstance: IObject {
    fn get_instance_component(&self) -> &InstanceComponent;
    fn get_instance_component_mut(&mut self) -> &mut InstanceComponent;
    
    fn lua_set(&mut self, lua: &Lua, name: String, val: LuaValue) -> LuaResult<()>;

    fn clone_instance(&self) -> LuaResult<ManagedInstance>;
}

impl dyn IInstance {
    pub fn get_parent(&self) -> Option<ManagedInstance> {
        let parent = self.get_instance_component().parent.as_ref();
        if parent.is_some() { unsafe {
            let parent = parent.unwrap_unchecked().upgrade();
            if parent.is_some() {
                return Some(parent.unwrap_unchecked())
            }
            return None
        }}
        None
    }
    pub fn set_parent(&mut self, parent: Option<ManagedInstance>) -> LuaResult<()> {
        if self.get_parent_protected() {
            Err(LuaError::RuntimeError("Parent is protected and cannot be set.".into()))
        } else {
            todo!();
            self.get_instance_component_mut().parent = parent.map(|x| x.downgrade());
            Ok(())
        }
    }

    pub fn get_name(&self) -> String {
        self.get_instance_component().name.clone()
    }
    pub fn set_name(&mut self, val: String) -> LuaResult<()> {
        self.get_instance_component_mut().name = val;
        Ok(())
    }

    pub fn get_archivable(&self) -> bool {
        self.get_instance_component().archivable
    }
    pub fn set_archivable(&mut self, val: bool) -> LuaResult<()> {
        self.get_instance_component_mut().archivable = val;
        Ok(())
    }

    //pub fn get_class_name(&self) -> &'static str { self.get_instance_component().class_name }
    pub fn get_uniqueid(&self) -> usize {
        self.get_instance_component().unique_id
    }
    pub fn init_uniqueid(&mut self) -> LuaResult<()> {
        let component = self.get_instance_component_mut();
        if component.unique_id != 0 {
            return Err(LuaError::RuntimeError("Instance::UniqueId was previously initialized.".into()));
        }
        component.unique_id = (&raw const *component) as usize;
        Ok(())
    }
    pub fn set_uniqueid(&mut self, value: usize) -> LuaResult<()> {
        let component = self.get_instance_component_mut();
        if component.unique_id != 0 {
            return Err(LuaError::RuntimeError("Instance::UniqueId was previously initialized.".into()));
        }
        component.unique_id = value;
        Ok(())
    }
    pub fn get_actor(&self) -> LuaResult<Option<ManagedInstance>> {
        self.find_first_ancestor_of_class("Actor".into())
    }
    pub fn get_debug_id(&self, scope_length: LuaNumber) -> LuaResult<String> {
        Err(LuaError::RuntimeError("Woopsies :3".into()))
    }
    pub fn get_descendants(&self) -> LuaResult<Vec<ManagedInstance>> {
        let mut current = self.get_children()?;
        let mut descendants: Vec<ManagedInstance> = vec![];
        descendants.append(&mut current);
        while current.len() != 0 {
            let mut new_current: Vec<ManagedInstance> = vec![];
            for i in current {
                let read = i.read().unwrap();
                new_current.append(&mut read.get_descendants()?);
            }
            descendants.append(&mut new_current);
            current = new_current;
        };
        Ok(descendants)
    }
    pub fn get_full_name(&self) -> LuaResult<String> {
        let mut hierarchy = Vec::new();
        let mut parent = self.get_parent();
        while parent.is_some() {
            let parent_unwrapped = unsafe { parent.unwrap_unchecked() };
            let parent_read = parent_unwrapped.read().unwrap();
            hierarchy.insert(0, parent_read.get_name());

            parent = parent_read.get_parent();
        }
        hierarchy.push(self.get_name());
        Ok(hierarchy.join("."))
    }
    pub fn get_parent_protected(&self) -> bool {
        self.get_instance_component().parent_locked
    }
    pub fn lock_parent(&mut self) {
        self.get_instance_component_mut().parent_locked = true;
    }
    pub fn add_tag(&mut self, tag: String) -> LuaResult<()> {
        todo!()
    }
    pub fn clear_all_children(&mut self) -> LuaResult<()> {
        let component = self.get_instance_component_mut();
        let mut children_removed = Vec::new();
        component.children_cache_dirty = true;
        for (idx, i) in component.children.iter().enumerate() {
            let mut instance_write = i.write().unwrap();
            if !instance_write.get_parent_protected() {
                instance_write.get_instance_component_mut().parent = None;
                // SAFETY: As parent got previously set to null, there will be no waiting on same lock.
                instance_write.destroy().unwrap();
                children_removed.push(idx-children_removed.len());
            }
        }
        for i in children_removed {
            component.children.remove(i);
        }
        Ok(())
    }
    pub async fn wait_for_child(&self, name: String, timeout: LuaNumber) -> LuaResult<ManagedInstance> {
        todo!()
    }
    pub fn find_first_ancestor(&self, name: String) -> LuaResult<Option<ManagedInstance>> {
        todo!()
    }
    pub fn find_first_ancestor_of_class(&self, class: String) -> LuaResult<Option<ManagedInstance>> {
        todo!()
    }
    pub fn find_first_ancestor_which_is_a(&self, class: String) -> LuaResult<Option<ManagedInstance>> {
        todo!()
    }
    pub fn find_first_child(&self, name: String) -> LuaResult<Option<ManagedInstance>> {
        todo!()
    }
    pub fn find_first_child_of_class(&self, class: String) -> LuaResult<Option<ManagedInstance>> {
        todo!()
    }
    pub fn find_first_child_which_is_a(&self, class: String) -> LuaResult<Option<ManagedInstance>> {
        todo!()
    }
    pub fn find_first_descendant(&self, name: String) -> LuaResult<Option<ManagedInstance>> {
        todo!()
    }
    pub fn get_attribute(&self, attribute: String) -> LuaResult<LuaValue> {
        todo!()
    }
    pub fn get_attribute_changed_signal(&self, attribute: String) -> LuaResult<ManagedRBXScriptSignal> {
        todo!()
    }
    pub fn get_attributes(&self, attribute: String) -> LuaResult<LuaTable> {
        todo!()
    }
    pub fn get_children(&self) -> LuaResult<Vec<ManagedInstance>> {
        todo!()
    }
    pub fn get_tags(&self) -> LuaResult<Vec<String>> {
        todo!()
    }
    pub fn has_tag(&self, tag: String) -> LuaResult<bool> {
        todo!()
    }
    pub fn is_ancestor_of(&self, descendant: ManagedInstance) -> LuaResult<bool> {
        todo!()
    }
    pub fn is_descendant_of(&self, ancestor: ManagedInstance) -> LuaResult<bool> {
        todo!()
    }
    pub fn remove_tag(&self, tag: String) -> LuaResult<()> {
        todo!()
    }
    pub fn set_attribute(&self, attribute: String, value: LuaValue) -> LuaResult<()> {
        todo!()
    }
    pub fn destroy(&mut self) -> LuaResult<()> {
        todo!()
    }

}

impl Debug for dyn IInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:x}", &raw const *self as *const u8 as usize))
    }
}

#[derive(Debug)]
pub struct InstanceComponent {
    archivable: bool,
    name: String,
    parent: Option<WeakManagedInstance>,
    _ptr: Option<WeakManagedInstance>,
    unique_id: usize,
    children: Vec<ManagedInstance>,
    children_cache: HashMap<String, WeakManagedInstance>,
    children_cache_dirty: bool,
    parent_locked: bool,

    pub ancestry_changed: ManagedRBXScriptSignal,
    pub attribute_changed: ManagedRBXScriptSignal,
    pub child_added: ManagedRBXScriptSignal,
    pub child_removed: ManagedRBXScriptSignal,
    pub descendant_added: ManagedRBXScriptSignal,
    pub descendant_removing: ManagedRBXScriptSignal,
    pub destroying: ManagedRBXScriptSignal,

    pub attribute_changed_table: EventsTable,
    pub property_changed_table: EventsTable
}

impl PartialEq for dyn IInstance {
    fn eq(&self, other: &Self) -> bool {
        (&raw const *self) == (&raw const *other)
    }
}
impl Eq for dyn IInstance {}

impl IInstanceComponent for InstanceComponent {
    fn lua_get(&self, _ptr: WeakManagedInstance, lua: &Lua, key: &String) -> Option<LuaResult<LuaValue>> {
        Some(InstanceComponent::lua_get(self, lua, key))
    }

    fn lua_set(&mut self, _ptr: WeakManagedInstance, lua: &Lua, key: &String, value: &LuaValue) -> Option<LuaResult<()>> {
        Some(InstanceComponent::lua_set(self, lua, key, value))
    }

    fn new(ptr: WeakManagedInstance, class_name: &'static str) -> Self {
        let mut inst = InstanceComponent {
            parent: None,
            name: String::from(class_name),
            archivable: true,
            unique_id: usize::default(), //uninitialized,
            _ptr: Some(ptr),
            parent_locked: false,
            children: Vec::new(),
            children_cache: HashMap::new(),
            children_cache_dirty: false,

            ancestry_changed: ManagedRBXScriptSignal::default(),
            attribute_changed: ManagedRBXScriptSignal::default(),
            child_added: ManagedRBXScriptSignal::default(),
            child_removed: ManagedRBXScriptSignal::default(),
            descendant_added: ManagedRBXScriptSignal::default(),
            descendant_removing: ManagedRBXScriptSignal::default(),
            destroying: ManagedRBXScriptSignal::default(),
            
            attribute_changed_table: EventsTable::default(),
            property_changed_table: EventsTable::default()
        };
        inst
    }
    fn clone(&self, ptr: WeakManagedInstance) -> LuaResult<Self> {
        let mut new_children = Vec::new();
        for i in self.children.iter() {
            let read = i.read();
            if read.is_err() {
                return Err(LuaError::RuntimeError("failed to acquire read lock on a child".into()));
            }
            let read = unsafe { read.unwrap_unchecked() };
            let inst = read.clone_instance();
            if inst.is_ok() {
                new_children.push(unsafe { inst.unwrap_unchecked() });
            }
        }
        Ok(InstanceComponent {
            archivable: self.archivable,
            name: self.name.clone(),
            parent: None,
            unique_id: 0,
            children: new_children,
            children_cache: HashMap::default(),
            children_cache_dirty: true,
            _ptr: Some(ptr),
            parent_locked: false,

            ancestry_changed: ManagedRBXScriptSignal::default(),
            attribute_changed: ManagedRBXScriptSignal::default(),
            child_added: ManagedRBXScriptSignal::default(),
            child_removed: ManagedRBXScriptSignal::default(),
            descendant_added: ManagedRBXScriptSignal::default(),
            descendant_removing: ManagedRBXScriptSignal::default(),
            destroying: ManagedRBXScriptSignal::default(),
            
            attribute_changed_table: EventsTable::default(),
            property_changed_table: EventsTable::default()
        })
    }

}

impl InstanceComponent {
    pub fn get_instance_pointer(&self) -> ManagedInstance {
        unsafe {
            self._ptr.as_ref().unwrap_unchecked().upgrade().unwrap_unchecked()
        }
    }
    pub fn get_weak_instance_pointer(&self) -> WeakManagedInstance {
        unsafe { self._ptr.as_ref().unwrap_unchecked().clone() }
    }
    pub fn lua_get(&self, lua: &Lua, key: &String) -> LuaResult<LuaValue> {
        match key.as_str() {
            "Archivable" => lua_getter!(lua, self.archivable),
            "ClassName" => IntoLua::into_lua(unsafe { 
                self._ptr.as_ref().unwrap_unchecked()
                    .upgrade().unwrap_unchecked().access()
                    .as_ref().unwrap_unchecked()
                    .get_class_name()
                }, lua),
            "Name" => lua_getter!(string, lua, self.name),
            "Parent" => lua_getter!(opt_weak_clone, lua, self.parent),

            "" => todo!(),
            _ => Ok(LuaNil)
        }
    }
    pub fn lua_set(&mut self, lua: &Lua, key: &String, value: &LuaValue) -> LuaResult<()> {
        match key.as_str() {
            "Archivable" => {
                self.archivable = value.as_boolean().ok_or(LuaError::RuntimeError("bad argument to setting Archivable".into()))?;
                todo!("event emit signal");
                Ok(())
            },
            "Name" => {
                self.name = value.as_string_lossy().ok_or(LuaError::RuntimeError("bad argument to setting Name".into()))?;
                todo!("event emit signal");
                Ok(())
            },
            "Parent" => {
                todo!();
            },
            _ => Err(LuaError::RuntimeError(format!("invalid property to set, {}",key.as_str())))
        }
    }
}

impl<T: IInstance, A: Allocator + Clone + Send + Sync> IWeak<T, A> {
    pub fn cast_to_instance(&self) -> IWeak<dyn IInstance, A> {
        let (header_raw, p_raw, alloc) = self.clone().into_inner_with_allocator();
        let p = p_raw as *mut dyn IInstance;
        let header = unsafe { NonNull::new_unchecked(header_raw.as_ptr() as *mut ITrcHead<dyn IInstance>) };
        IWeak::from_inner_with_allocator((header, p, alloc))
    }
}