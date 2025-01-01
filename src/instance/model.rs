use super::pvinstance::IPVInstance;
use super::{IInstance, IObject, InstanceComponent, ManagedInstance, PVInstanceComponent};

use crate::core::{ITrc, IWeak, InheritanceBase, InheritanceTable, InheritanceTableBuilder};
use crate::userdata::CFrame;
use crate::userdata::enums::{ModelLevelOfDetail, ModelStreamingMode};

pub struct ModelComponent {
    level_of_detail: ModelLevelOfDetail,
    model_streaming_mode: ModelStreamingMode,
    primary_part: Option<ManagedInstance>, // todo!()
    world_pivot: CFrame
}
pub struct Model {
    instance: InstanceComponent,
    pvinstance: PVInstanceComponent,
    model: ModelComponent
}
pub trait IModel: IPVInstance {}

/*impl InheritanceBase for Model {
    fn inheritance_table(&self) -> InheritanceTable {
        InheritanceTableBuilder::new()
            .insert_type::<Model,dyn IObject>(|x: &Self| x as &dyn IObject, |x: &mut Self| x as &mut dyn IObject)
            .insert_type::<Model,dyn IInstance>(|x: &Self| x as &dyn IInstance, |x: &mut Self| x as &mut dyn IInstance)
            .insert_type::<Model,dyn IPVInstance>(|x: &Self| x as &dyn IPVInstance, |x: &mut Self| x as &mut dyn IPVInstance)
            .insert_type::<Model,dyn IModel>(|x: &Self| x as &dyn IModel, |x: &mut Self| x as &mut dyn IModel)
            .output()
    }
}
impl IInstance for Model {
    
}*/