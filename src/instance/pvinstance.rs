use crate::userdata::CFrame;

use super::IInstance;

pub struct PVInstanceComponent {
    origin: CFrame,
    pivot_offset: CFrame
}

pub trait IPVInstance: IInstance {
    fn get_pv_instance_component(&self) -> &PVInstanceComponent;
    fn get_pv_instance_component_mut(&mut self) -> &mut PVInstanceComponent;
}

impl dyn IPVInstance {
    pub fn get_pivot(&self) -> CFrame {
        todo!()
    }
    pub fn pivot_to(&mut self, pivot: CFrame)  {
        todo!()
    }
}

