use std::{any::{Any, TypeId}, collections::HashMap, mem::transmute};

pub struct InheritanceCastError;

#[doc(hidden)]
pub struct InheritanceTable {
    // Function pointers. Their actual return values and arguments are not known inside the struct.
    v: HashMap<TypeId, (fn() -> (), fn() -> ())>
}
#[derive(Default, Debug)]
pub struct InheritanceTableBuilder {
    v: HashMap<TypeId, (fn() -> (), fn() -> ())>
}

impl InheritanceTable {
    #[doc(hidden)]
    fn index<'a, T: 'static>(&self) -> Result<(fn(&'a dyn InheritanceBase) -> &'a T, fn(&'a mut dyn InheritanceBase) -> &'a mut T),InheritanceCastError> {
        let typeid: TypeId = TypeId::of::<T>();
        match self.v.get(&typeid) {
            None => Err(InheritanceCastError),
            Some(p) => {
                let new_ptrs: (fn(&'a dyn InheritanceBase) -> &'a T, fn(&'a mut dyn InheritanceBase) -> &'a mut T) = unsafe { transmute(*p) };
                Ok(new_ptrs)
            }
        }
    }
    #[doc(hidden)]
    fn has<T: 'static>(&self) -> bool {
        let typeid: TypeId = TypeId::of::<T>();
        self.v.contains_key(&typeid)
    }
}

pub trait InheritanceBase: Any {
    #[allow(private_interfaces)]
    fn inheritance_table(&self) -> InheritanceTable;
}

impl dyn InheritanceBase {
    pub fn inherit_as<T: 'static>(&self) -> Result<&T, InheritanceCastError> {
        let ptrs = self.inheritance_table().index::<T>()?;
        Ok(ptrs.0(self))
    }
    pub fn inherit_as_mut<T: 'static>(&mut self) -> Result<&mut T, InheritanceCastError> {
        let ptrs = self.inheritance_table().index::<T>()?;
        Ok(ptrs.1(self))
    }
    pub fn is<T: 'static>(&self) -> bool {
        self.inheritance_table().has::<T>()
    }
}

impl InheritanceTableBuilder {
    pub fn new() -> Self {
        Self {
            v: HashMap::new()
        }
    }
    pub fn insert_type<'a, T: 'static + InheritanceBase, To: 'static + ?Sized>(mut self, inherit: fn(&'a T) -> &'a To, inherit_mut: fn(&'a mut T) -> &'a mut To) -> Self {
        let ptrs = (
            |x: &'a dyn InheritanceBase| -> &'a To {
                let this = unsafe { & *(x as *const dyn InheritanceBase as *const T) };
                inherit(this)
            },
            |x: &'a mut dyn InheritanceBase| -> &'a mut To {
                let this = unsafe { &mut *(x as *mut dyn InheritanceBase as *mut T) };
                inherit_mut(this)
            },
        );
        self.v.insert(TypeId::of::<T>(), unsafe { transmute(ptrs) });
        self
    }
    #[allow(private_interfaces)]
    pub fn output(self) -> InheritanceTable {
        InheritanceTable {
            v: self.v
        }
    }
}