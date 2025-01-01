use std::collections::HashMap;

use crate::core::{InheritanceBase, ITrc, IWeak};
use crate::userdata::{ManagedRBXScriptSignal, RBXScriptSignal};
use mlua::prelude::*;

use super::IObject;

pub type ManagedInstance = ITrc<dyn IInstance>;
pub type WeakManagedInstance = IWeak<dyn IInstance>;
pub type EventsTable = HashMap<String, ManagedRBXScriptSignal>;

macro_rules! lua_getter {
    ($lua: ident, $prop: expr) => {
        IntoLua::into_lua($prop, $lua)
    };
    (string, $lua: ident, $prop: expr) => {
        IntoLua::into_lua($prop.as_str(), $lua)
    };
    (clone, $lua: ident, $prop: expr) => {
        IntoLua::into_lua($prop.clone(), $lua)
    };
    (opt_clone, $lua: ident, $prop: expr) => {
        IntoLua::into_lua($prop.as_ref(), $lua)
    };
    (weak_clone, $lua: ident, $prop: expr) => {
        IntoLua::into_lua($prop.upgrade(), $lua)
    };
    (opt_weak_clone, $lua: ident, $prop: expr) => {
        IntoLua::into_lua($prop.as_ref().map(|x| x.upgrade()).flatten(), $lua)
    };
}
macro_rules! lua_setter {
    ($lua: ident, $prop: expr, $value: ident) => {
        todo!()
    };
}

pub trait IInstance: InheritanceBase {
    fn get_instance_component(&self) -> &InstanceComponent;
    fn get_instance_component_mut(&mut self) -> &mut InstanceComponent;
    
    fn lua_get(&self, lua: &Lua, name: String) -> LuaResult<LuaValue>;
    fn lua_set(&mut self, lua: &Lua, name: String, val: LuaValue) -> LuaResult<()>;

    fn clone_instance(&self) -> LuaResult<ManagedInstance>;
    
    fn get_property_changed_signal(&self, property: String) -> RBXScriptSignal {
        todo!()
    }
    fn get_changed_signal(&self) -> RBXScriptSignal {
        todo!()
    }
    fn is_a(&self, class_name: String) -> bool {
        match class_name.as_str() {
            "Instance" |
            "Object" => true,
            _ => false
        }
    }
}

impl IObject for dyn IInstance {
    fn is_a(&self, class_name: String) -> bool {
        match class_name.as_str() {
            "Object" => true,
            _ => false
        }
    }
    
    fn lua_get(&self, lua: &Lua, name: String) -> LuaResult<LuaValue> {
        IInstance::lua_get(self, lua, name)
    }
    
    fn get_class_name(&self) -> &'static str {
        self.get_instance_component().class_name
    }
    
    fn get_property_changed_signal(&self, property: String) -> RBXScriptSignal {
        IInstance::get_property_changed_signal(self, property)
    }
    
    fn get_changed_signal(&self) -> RBXScriptSignal {
        IInstance::get_changed_signal(self)
    }
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

    pub fn get_class_name(&self) -> &'static str { self.get_instance_component().class_name }
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
    pub fn get_events_mut(&mut self) -> &mut EventsTable {
        &mut self.get_instance_component_mut().events
    }
    pub fn get_events(&self) -> &EventsTable {
        &self.get_instance_component().events
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
pub struct InstanceComponent {
    pub events: EventsTable,
    archivable: bool,
    class_name: &'static str,
    name: String,
    parent: Option<WeakManagedInstance>,
    _ptr: Option<WeakManagedInstance>,
    unique_id: usize,
    children: Vec<ManagedInstance>,
    children_cache: HashMap<String, WeakManagedInstance>,
    children_cache_dirty: bool,
    parent_locked: bool
}

impl PartialEq for dyn IInstance {
    fn eq(&self, other: &Self) -> bool {
        (&raw const *self) == (&raw const *other)
    }
}
impl Eq for dyn IInstance {}

impl InstanceComponent {
    pub fn get_instance_pointer(&self) -> ManagedInstance {
        unsafe {
            self._ptr.as_ref().unwrap_unchecked().upgrade().unwrap_unchecked()
        }
    }
    pub fn get_weak_instance_pointer(&self) -> WeakManagedInstance {
        unsafe { self._ptr.as_ref().unwrap_unchecked().clone() }
    }
    pub fn new(ptr: WeakManagedInstance, class_name: &'static str) -> InstanceComponent {
        let mut inst = InstanceComponent {
            parent: None,
            name: String::from(class_name),
            archivable: true,
            unique_id: usize::default(), //uninitialized,
            events: EventsTable::default(), //uninit
            class_name: class_name,
            _ptr: Some(ptr),
            parent_locked: false,
            children: Vec::new(),
            children_cache: HashMap::new(),
            children_cache_dirty: false
        };
        inst.events.insert("AncestryChanged".into(),
            ManagedRBXScriptSignal::new(RBXScriptSignal::default())
        );
        inst.events.insert("AttributeChanged".into(),
            ManagedRBXScriptSignal::new(RBXScriptSignal::default())
        );
        inst.events.insert("ChildAdded".into(), 
            ManagedRBXScriptSignal::new(RBXScriptSignal::default())
        );
        inst.events.insert("ChildRemoved".into(), 
            ManagedRBXScriptSignal::new(RBXScriptSignal::default())
        );
        inst.events.insert("DescendantAdded".into(), 
            ManagedRBXScriptSignal::new(RBXScriptSignal::default())
        );
        inst.events.insert("DescendantRemoving".into(), 
            ManagedRBXScriptSignal::new(RBXScriptSignal::default())
        );
        inst.events.insert("Destroying".into(), 
            ManagedRBXScriptSignal::new(RBXScriptSignal::default())
        );
        inst
    }
    pub fn lua_get(&self, lua: &Lua, key: &String) -> LuaResult<LuaValue> {
        match key.as_str() {
            "Archivable" => lua_getter!(lua, self.archivable),
            "ClassName" => lua_getter!(lua, self.class_name),
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
