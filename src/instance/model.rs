use super::instance::IInstanceComponent;
use super::pvinstance::IPVInstance;
use super::{IInstance, IObject, InstanceComponent, ManagedInstance, PVInstanceComponent};

use crate::core::{ITrc, IWeak, InheritanceBase, InheritanceTable, InheritanceTableBuilder};
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
    instance: InstanceComponent,
    pvinstance: PVInstanceComponent,
    model: ModelComponent
}
pub trait IModel: IPVInstance {
    fn get_model_component(&self) -> &ModelComponent;
    fn get_model_component_mut(&mut self) -> &mut ModelComponent;
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
    fn is_a(&self, class_name: String) -> bool {
        match class_name.as_str() {
            "Model" |
            "PVInstance" |
            "Instance" |
            "Object" => true,
            _ => false
        }
    }
    fn lua_get(&self, lua: &mlua::Lua, name: String) -> mlua::Result<mlua::Value> {
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
    fn get_instance_component(&self) -> &InstanceComponent {
        &self.instance
    }
    fn get_instance_component_mut(&mut self) -> &mut InstanceComponent {
        &mut self.instance
    }
    fn lua_set(&mut self, lua: &mlua::Lua, name: String, val: mlua::Value) -> mlua::Result<()> {
        todo!()
    }
    fn clone_instance(&self) -> mlua::Result<ManagedInstance> {
        todo!()
    }
}
impl IPVInstance for Model {
    fn get_pv_instance_component(&self) -> &PVInstanceComponent {
        &self.pvinstance
    }

    fn get_pv_instance_component_mut(&mut self) -> &mut PVInstanceComponent {
        &mut self.pvinstance
    }
}
impl IModel for Model {
    fn get_model_component(&self) -> &ModelComponent {
        &self.model
    }
    fn get_model_component_mut(&mut self) -> &mut ModelComponent {
        &mut self.model
    }
}

impl IInstanceComponent for ModelComponent {
    fn lua_get(&self, ptr: super::WeakManagedInstance, lua: &mlua::Lua, key: &String) -> Option<mlua::Result<mlua::Value>> {
        todo!()
    }

    fn lua_set(&mut self, ptr: super::WeakManagedInstance, lua: &mlua::Lua, key: &String, value: &mlua::Value) -> Option<mlua::Result<()>> {
        todo!()
    }

    fn clone(&self, new_ptr: super::WeakManagedInstance) -> mlua::Result<Self> {
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
        ITrc::new_cyclic(|x| {
            Model {
                instance: InstanceComponent::new(x.cast_to_instance(), "Model"),
                pvinstance: PVInstanceComponent::new(x.cast_to_instance(), "Model"),
                model: ModelComponent::new(x.cast_to_instance(), "Model")
            }
        }).cast_sized().unwrap()
    }
}