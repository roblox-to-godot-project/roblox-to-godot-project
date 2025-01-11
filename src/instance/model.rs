use mlua::prelude::*;

use super::instance::IInstanceComponent;
use super::pvinstance::IPVInstance;
use super::{IInstance, IObject, InstanceComponent, ManagedInstance, PVInstanceComponent};

use crate::core::{IWeak, InheritanceBase, InheritanceTable, InheritanceTableBuilder, Irc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::userdata::CFrame;
use crate::userdata::enums::{ModelLevelOfDetail, ModelStreamingMode};

#[derive(Debug)]
pub struct ModelComponent {
    level_of_detail: ModelLevelOfDetail,
    model_streaming_mode: ModelStreamingMode,
    primary_part: Option<ManagedInstance>, // todo!()
    world_pivot: CFrame
}
#[derive(Debug)]
pub struct Model {
    instance: RwLock<InstanceComponent>,
    pvinstance: RwLock<PVInstanceComponent>,
    model: RwLock<ModelComponent>
}
pub trait IModel: IPVInstance {
    fn get_model_component(&self) -> RwLockReadGuard<'_,ModelComponent>;
    fn get_model_component_mut(&self) -> RwLockWriteGuard<'_,ModelComponent>;
}

impl InheritanceBase for Model {
    fn inheritance_table(&self) -> InheritanceTable {
        InheritanceTableBuilder::new()
            .insert_type::<Model,dyn IObject>(|x: &Self| x as &dyn IObject, |x: &mut Self| x as &mut dyn IObject)
            .insert_type::<Model,dyn IInstance>(|x: &Self| x as &dyn IInstance, |x: &mut Self| x as &mut dyn IInstance)
            .insert_type::<Model,dyn IPVInstance>(|x: &Self| x as &dyn IPVInstance, |x: &mut Self| x as &mut dyn IPVInstance)
            .insert_type::<Model,dyn IModel>(|x: &Self| x as &dyn IModel, |x: &mut Self| x as &mut dyn IModel)
            .output()
    }
}
impl IObject for Model {
    fn is_a(&self, class_name: &String) -> bool {
        match class_name.as_str() {
            "Model" |
            "PVInstance" |
            "Instance" |
            "Object" => true,
            _ => false
        }
    }
    fn lua_get(&self, lua: &Lua, name: String) -> LuaResult<LuaValue> {
        todo!()
    }
    fn get_changed_signal(&self) -> crate::userdata::RBXScriptSignal {
        todo!()
    }
    fn get_property_changed_signal(&self, property: String) -> crate::userdata::RBXScriptSignal {
        todo!()
    }
    fn get_class_name(&self) -> &'static str { "Model" }
}
impl IInstance for Model {
    fn get_instance_component(&self) -> RwLockReadGuard<'_, InstanceComponent> {
        self.instance.read().unwrap()
    }
    fn get_instance_component_mut(&self) -> RwLockWriteGuard<'_, InstanceComponent> {
        self.instance.write().unwrap()
    }
    fn lua_set(&self, lua: &Lua, name: String, val: LuaValue) -> LuaResult<()> {
        todo!()
    }
    fn clone_instance(&self) -> LuaResult<ManagedInstance> {
        todo!()
    }
}
impl IPVInstance for Model {
    fn get_pv_instance_component(&self) -> RwLockReadGuard<'_, PVInstanceComponent> {
        self.pvinstance.read().unwrap()
    }

    fn get_pv_instance_component_mut(&self) -> RwLockWriteGuard<'_, PVInstanceComponent> {
        self.pvinstance.write().unwrap()
    }
}
impl IModel for Model {
    fn get_model_component(&self) -> RwLockReadGuard<'_, ModelComponent> {
        self.model.read().unwrap()
    }
    fn get_model_component_mut(&self) -> RwLockWriteGuard<'_, ModelComponent> {
        self.model.write().unwrap()
    }
}

impl IInstanceComponent for ModelComponent {
    fn lua_get(self: &mut RwLockReadGuard<'_, ModelComponent>, ptr: super::WeakManagedInstance, lua: &Lua, key: &String) -> Option<LuaResult<LuaValue>> {
        todo!()
    }

    fn lua_set(self: &mut RwLockWriteGuard<'_, ModelComponent>, ptr: super::WeakManagedInstance, lua: &Lua, key: &String, value: &LuaValue) -> Option<LuaResult<()>> {
        todo!()
    }

    fn clone(self: &RwLockReadGuard<'_, ModelComponent>, new_ptr: super::WeakManagedInstance) -> LuaResult<Self> {
        todo!()
    }

    fn new(ptr: super::WeakManagedInstance, class_name: &'static str) -> Self {
        ModelComponent {
            level_of_detail: ModelLevelOfDetail::Automatic,
            model_streaming_mode: ModelStreamingMode::Default,
            primary_part: None,
            world_pivot: CFrame::IDENTITY
        }
    }
}

impl Model {
    pub fn new() -> ManagedInstance {
        Irc::new_cyclic(|x| {
            Model {
                instance: RwLock::new(InstanceComponent::new(x.cast_to_instance(), "Model")),
                pvinstance: RwLock::new(PVInstanceComponent::new(x.cast_to_instance(), "Model")),
                model: RwLock::new(ModelComponent::new(x.cast_to_instance(), "Model"))
            }
        }).cast_from_sized().unwrap()
    }
}