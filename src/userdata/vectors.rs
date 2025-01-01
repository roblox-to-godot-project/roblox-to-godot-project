use std::ops::{Add, Div, Mul, Sub};
use std::sync::Arc;

use godot::builtin::math::{ApproxEq, FloatExt};
use mlua::prelude::*;

use super::enums::{Axis, NormalId};
use super::LuaSingleton;

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
pub struct Vector2<T = f64> {
    pub x: T,
    pub y: T
}
pub type Vector2int16 = Vector2<i16>;

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug)]
pub struct Vector3<T = f64> {
    pub x: T,
    pub y: T,
    pub z: T
}
pub type Vector3int16 = Vector3<i16>;

impl Vector2 {
    pub const ZERO: Vector2 = Vector2 {x: 0f64, y: 0f64};
    pub const ONE: Vector2 = Vector2 {x: 1f64, y: 1f64};
    pub const X_AXIS: Vector2 = Vector2 {x: 1f64, y: 0f64};
    pub const Y_AXIS: Vector2 = Vector2 {x: 0f64, y: 1f64};

    pub const fn new(x: f64, y: f64) -> Vector2 {
        Vector2 {
            x, y
        }
    }

    pub fn get_magnitude(&self) -> f64 {
        (self.x*self.x + self.y*self.y).sqrt()
    }
    pub fn set_magnitude(&mut self, value: f64) {
        *self = self.get_unit()*value;
    }
    pub fn get_unit(&self) -> Vector2 {
        let magnitude = self.get_magnitude();
        Vector2 {
            x: self.x / magnitude,
            y: self.y / magnitude
        }
    }
    pub fn cross(&self, other: Vector2) -> f64 {
        self.x*other.y - self.y*other.x
    }
    pub fn abs(&self) -> Vector2 {
        Vector2 {
            x: self.x.abs(),
            y: self.y.abs()
        }
    }
    pub fn ceil(&self) -> Vector2 {
        Vector2 {
            x: self.x.ceil(),
            y: self.y.ceil()
        }
    }
    pub fn floor(&self) -> Vector2 {
        Vector2 {
            x: self.x.floor(),
            y: self.y.floor()
        }
    }
    pub fn sign(&self) -> Vector2 {
        Vector2 {
            x: self.x.sign(),
            y: self.y.sign()
        }
    }
    pub fn dot(&self, other: Vector2) -> f64 {
        self.x*other.x+self.y*other.y
    }
    pub fn get_angle(&self, other: Vector2, is_signed: bool) -> f64 {
        if is_signed {
            todo!();
        }
        (self.dot(other)/(self.get_magnitude()*other.get_magnitude())).acos()
    }
    pub fn lerp(&self, other: Vector2, alpha: f64) -> Vector2 {
        *self+(other-*self)*alpha
    }
    pub fn max(&self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y)
        }
    }
    pub fn min(&self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y)
        }
    }
    pub fn fuzzy_eq(&self, other: Vector2) -> bool {
        self.x.approx_eq(&other.x) && self.y.approx_eq(&other.y)
    }
}

impl Vector2int16 {
    pub const fn new(x: i16, y: i16) -> Vector2int16 {
        Vector2int16 {
            x, y
        }
    }
}

impl<T> Add for Vector2<T>
where
    T: Add<T, Output = T> + Copy
{
    type Output = Vector2<T>;
    
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x+rhs.x,
            y: self.y+rhs.y
        }
    }
}
impl<T> Sub for Vector2<T>
where
    T: Sub<T, Output = T> + Copy
{
    type Output = Vector2<T>;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x-rhs.x,
            y: self.y-rhs.y
        }
    }
}
impl<T> Mul for Vector2<T>
where
    T: Mul<T, Output = T> + Copy
{
    type Output = Vector2<T>;
    
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x*rhs.x,
            y: self.y*rhs.y
        }
    }
}
impl<T> Div for Vector2<T>
where
    T: Div<T, Output = T> + Copy
{
    type Output = Vector2<T>;
    
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x/rhs.x,
            y: self.y/rhs.y
        }
    }
}
impl Mul<f64> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x*rhs,
            y: self.y*rhs
        }
    }
}
impl Div<f64> for Vector2 {
    type Output = Vector2;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x/rhs,
            y: self.y/rhs
        }
    }
}

impl Mul<f64> for Vector2int16 {
    type Output = Vector2int16;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x*rhs as i16,
            y: self.y*rhs as i16
        }
    }
}
impl Div<f64> for Vector2int16 {
    type Output = Vector2int16;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x/rhs as i16,
            y: self.y/rhs as i16
        }
    }
}
impl Mul<i64> for Vector2int16 {
    type Output = Vector2int16;
    fn mul(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x*rhs as i16,
            y: self.y*rhs as i16
        }
    }
}
impl Div<i64> for Vector2int16 {
    type Output = Vector2int16;
    fn div(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x/rhs as i16,
            y: self.y/rhs as i16
        }
    }
}

from_lua_copy_impl!(Vector2);
from_lua_copy_impl!(Vector2int16);

impl LuaUserData for Vector2 {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("X", |_, this| Ok(this.x));
        fields.add_field_method_set("X", |_, this, v| Ok({this.x = v;}));
        
        fields.add_field_method_get("Y", |_, this| Ok(this.y));
        fields.add_field_method_set("Y", |_, this, v| Ok({this.y = v;}));

        fields.add_field_method_get("Unit", |_, this| Ok(this.get_unit()));

        fields.add_field_method_get("Magnitude", |_, this| Ok(this.get_magnitude()));
        fields.add_field_method_set("Magnitude", |_, this, val| Ok(this.set_magnitude(val)));
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Cross",|_, this, other| Ok(this.cross(other)));
        methods.add_method("Abs", |_, this, ()| Ok(this.abs()));
        methods.add_method("Ceil", |_, this, ()| Ok(this.ceil()));
        methods.add_method("Floor", |_, this, ()| Ok(this.floor()));
        methods.add_method("Sign", |_, this, ()| Ok(this.sign()));
        methods.add_method("Angle", 
            |_, this, (other, is_signed): (Vector2, Option<bool>)| 
                Ok(this.get_angle(other, is_signed.unwrap_or(false)))
        );
        methods.add_method("Dot",|_, this, other| Ok(this.dot(other)));
        methods.add_method("Lerp",|_, this, (other, alpha)| Ok(this.lerp(other, alpha)));
        methods.add_method("Max",|_, this, other| Ok(this.max(other)));
        methods.add_method("Min",|_, this, other| Ok(this.min(other)));
        methods.add_method("FuzzyEq",|_, this, other: Vector2 | {
            todo!();
            Ok(())
        });
        methods.add_meta_method("__add", |_, this, other| Ok(*this+other));
        methods.add_meta_method("__sub", |_, this, other| Ok(*this-other));
        methods.add_meta_method("__mul", |lua, this, val: LuaValue| {
            if val.is_number() {
                Ok(*this * unsafe { val.as_number().unwrap_unchecked() })
            } else if val.is_userdata() {
                let result = Vector2::from_lua(val, lua);
                if result.is_ok() {
                    Ok(*this * unsafe { result.unwrap_unchecked() })
                } else {
                    Err(LuaError::BadArgument {
                        to: Some("Vector2::__mul".into()),
                        pos: 2,
                        name: Some("other".into()),
                        cause: Arc::new(result.unwrap_err())
                    })
                }
            } else {
                Err(LuaError::BadArgument {
                    to: Some("Vector2::__mul".into()),
                    pos: 2,
                    name: Some("other".into()),
                    cause: Arc::new(LuaError::FromLuaConversionError {
                        from: val.type_name(),
                        to: "Vector2".into(),
                        message: None
                    })})
            }
        });
        methods.add_meta_method("__div", |lua, this, val: LuaValue| {
            if val.is_number() {
                Ok(*this / unsafe { val.as_number().unwrap_unchecked() })
            } else if val.is_userdata() {
                let result = Vector2::from_lua(val, lua);
                if result.is_ok() {
                    Ok(*this / unsafe { result.unwrap_unchecked() })
                } else {
                    Err(LuaError::BadArgument {
                        to: Some("Vector2::__div".into()),
                        pos: 2,
                        name: Some("other".into()),
                        cause: Arc::new(result.unwrap_err())
                    })
                }
            } else {
                Err(LuaError::BadArgument {
                    to: Some("Vector2::__div".into()),
                    pos: 2,
                    name: Some("other".into()),
                    cause: Arc::new(LuaError::FromLuaConversionError {
                        from: val.type_name(),
                        to: "Vector2".into(),
                        message: None
                    })})
            }
        });
        methods.add_meta_method("__tostring", |_, this, ()| Ok(format!("({}, {})",this.x,this.y)));
        methods.add_meta_method("__eq", |_, this, other| Ok(*this == other));
    }
}

impl LuaSingleton for Vector2 {
    fn register_singleton(lua: &Lua) -> LuaResult<()> {
        let table = lua.create_table()?;
        table.raw_set(
            "new",
            lua.create_function(|_, (x, y): (f64, f64)| {
                Ok(Vector2 {
                    x, y
                })
            })?
        )?;
        table.raw_set("zero", Vector2::ZERO)?;
        table.raw_set("one", Vector2::ONE)?;
        table.raw_set("xAxis", Vector2::X_AXIS)?;
        table.raw_set("yAxis", Vector2::Y_AXIS)?;
        lua.globals().raw_set("Vector2", table)?;
        Ok(())
    }
}

impl LuaUserData for Vector2int16 {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("X", |_, this| Ok(this.x));
        fields.add_field_method_set("X", |_, this, v| Ok({this.x = v;}));
        
        fields.add_field_method_get("Y", |_, this| Ok(this.y));
        fields.add_field_method_set("Y", |_, this, v| Ok({this.y = v;}));
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__add", |_, this, other| Ok(*this+other));
        methods.add_meta_method("__sub", |_, this, other| Ok(*this-other));
        methods.add_meta_method("__mul", |lua, this, val: LuaValue| {
            if val.is_number() {
                Ok(*this * unsafe { val.as_number().unwrap_unchecked() })
            } else if val.is_userdata() {
                let result = Vector2int16::from_lua(val, lua);
                if result.is_ok() {
                    Ok(*this * unsafe { result.unwrap_unchecked() })
                } else {
                    Err(LuaError::BadArgument {
                        to: Some("Vector2int16::__mul".into()),
                        pos: 2,
                        name: Some("other".into()),
                        cause: Arc::new(result.unwrap_err())
                    })
                }
            } else {
                Err(LuaError::BadArgument {
                    to: Some("Vector2int16::__mul".into()),
                    pos: 2,
                    name: Some("other".into()),
                    cause: Arc::new(LuaError::FromLuaConversionError {
                        from: val.type_name(),
                        to: "Vector2int16".into(),
                        message: None
                    })})
            }
        });
        methods.add_meta_method("__div", |lua, this, val: LuaValue| {
            if val.is_number() {
                Ok(*this / unsafe { val.as_number().unwrap_unchecked() })
            } else if val.is_userdata() {
                let result = Vector2int16::from_lua(val, lua);
                if result.is_ok() {
                    Ok(*this / unsafe { result.unwrap_unchecked() })
                } else {
                    Err(LuaError::BadArgument {
                        to: Some("Vector2int16::__div".into()),
                        pos: 2,
                        name: Some("other".into()),
                        cause: Arc::new(result.unwrap_err())
                    })
                }
            } else {
                Err(LuaError::BadArgument {
                    to: Some("Vector2int16::__div".into()),
                    pos: 2,
                    name: Some("other".into()),
                    cause: Arc::new(LuaError::FromLuaConversionError {
                        from: val.type_name(),
                        to: "Vector2int16".into(),
                        message: None
                    })})
            }
        });
        methods.add_meta_method("__tostring", |_, this, ()| Ok(format!("({}, {})",this.x,this.y)));
    }
}

impl LuaSingleton for Vector2int16 {
    fn register_singleton(lua: &Lua) -> LuaResult<()> {
        let table = lua.create_table()?;
        table.raw_set(
            "new",
            lua.create_function(|_, (x, y): (i64, i64)| {
                Ok(Vector2int16 {
                    x: x as i16,
                    y: y as i16
                })
            })?
        )?;
        lua.globals().raw_set("Vector2int16", table)?;
        Ok(())
    }
}

impl Vector3 {
    pub const ZERO: Vector3 = Vector3 { x: 0f64, y: 0f64, z: 0f64 };
    pub const ONE: Vector3 = Vector3 { x: 1f64, y: 1f64, z: 1f64 };
    pub const X_AXIS: Vector3 = Vector3 { x: 1f64, y: 0f64, z: 0f64 };
    pub const Y_AXIS: Vector3 = Vector3 { x: 0f64, y: 1f64, z: 0f64 };
    pub const Z_AXIS: Vector3 = Vector3 { x: 0f64, y: 0f64, z: 1f64 };

    pub const fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 {
            x, y, z
        }
    }
    pub fn get_magnitude(&self) -> f64 {
        (self.x*self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub fn get_unit(&self) -> Vector3 {
        let magnitude = self.get_magnitude();
        Vector3 {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude
        }
    }
    pub fn set_magnitude(&mut self, magnitude: f64) {
        *self = self.get_unit()*magnitude;
    }
    pub fn abs(&self) -> Vector3 {
        Vector3 {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs()
        }
    }
    pub fn ceil(&self) -> Vector3 {
        Vector3 {
            x: self.x.ceil(),
            y: self.y.ceil(),
            z: self.z.ceil()
        }
    }
    pub fn floor(&self) -> Vector3 {
        Vector3 {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor()
        }
    }
    pub fn sign(&self) -> Vector3 {
        Vector3 {
            x: self.x.sign(),
            y: self.y.sign(),
            z: self.z.sign()
        }
    }
    pub fn cross(&self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.y*other.z - self.z*other.y,
            y: self.z*other.x - self.x*other.z,
            z: self.x*other.y - self.y*other.x
        }
    }
    pub fn get_angle(&self, other: Vector3, axis: Option<Vector3>) -> f64 {
        if axis.is_some() {
            todo!();
        }
        (self.dot(other)/(self.get_magnitude()*other.get_magnitude())).acos()
    }
    pub fn dot(&self, other: Vector3) -> f64 {
        self.x*other.x+self.y*other.y+self.z*other.z
    }
    pub fn fuzzy_eq(&self, other: Vector3) -> bool {
        self.x.approx_eq(&other.x)&&self.y.approx_eq(&other.y)&&self.z.approx_eq(&other.z)
    }
    pub fn lerp(&self, other: Vector3, alpha: f64) -> Vector3 {
        *self+(other-*self)*alpha
    }
    pub fn max(&self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z)
        }
    }
    pub fn min(&self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z)
        }
    }
}

from_lua_copy_impl!(Vector3);
from_lua_copy_impl!(Vector3int16);

impl<T> Add for Vector3<T>
where
    T: Add<T, Output = T> + Copy
{
    type Output = Vector3<T>;
    
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x+rhs.x,
            y: self.y+rhs.y,
            z: self.z+rhs.z
        }
    }
}
impl<T> Sub for Vector3<T>
where
    T: Sub<T, Output = T> + Copy
{
    type Output = Vector3<T>;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x-rhs.x,
            y: self.y-rhs.y,
            z: self.z-rhs.z
        }
    }
}
impl<T> Mul for Vector3<T>
where
    T: Mul<T, Output = T> + Copy
{
    type Output = Vector3<T>;
    
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x*rhs.x,
            y: self.y*rhs.y,
            z: self.z*rhs.z
        }
    }
}
impl<T> Div for Vector3<T>
where
    T: Div<T, Output = T> + Copy
{
    type Output = Vector3<T>;
    
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x/rhs.x,
            y: self.y/rhs.y,
            z: self.z/rhs.z
        }
    }
}
impl Mul<f64> for Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x*rhs,
            y: self.y*rhs,
            z: self.z*rhs
        }
    }
}
impl Div<f64> for Vector3 {
    type Output = Vector3;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x/rhs,
            y: self.y/rhs,
            z: self.z/rhs
        }
    }
}

impl Mul<f64> for Vector3int16 {
    type Output = Vector3int16;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x*rhs as i16,
            y: self.y*rhs as i16,
            z: self.z*rhs as i16
        }
    }
}
impl Div<f64> for Vector3int16 {
    type Output = Vector3int16;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x/rhs as i16,
            y: self.y/rhs as i16,
            z: self.z/rhs as i16
        }
    }
}
impl Mul<i64> for Vector3int16 {
    type Output = Vector3int16;
    fn mul(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x*rhs as i16,
            y: self.y*rhs as i16,
            z: self.z*rhs as i16
        }
    }
}
impl Div<i64> for Vector3int16 {
    type Output = Vector3int16;
    fn div(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x/rhs as i16,
            y: self.y/rhs as i16,
            z: self.z/rhs as i16
        }
    }
}

impl From<NormalId> for Vector3 {
    fn from(value: NormalId) -> Self {
        todo!()
    }
}
impl From<Axis> for Vector3 {
    fn from(value: Axis) -> Self {
        match value {
            Axis::X => Self::X_AXIS,
            Axis::Y => Self::Y_AXIS,
            Axis::Z => Self::Z_AXIS
        }
    }
}

impl LuaUserData for Vector3 {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("X", |_, this| Ok(this.x));
        fields.add_field_method_set("X", |_, this, v| Ok({this.x = v;}));
        
        fields.add_field_method_get("Y", |_, this| Ok(this.y));
        fields.add_field_method_set("Y", |_, this, v| Ok({this.y = v;}));

        fields.add_field_method_get("Z", |_, this| Ok(this.z));
        fields.add_field_method_set("Z", |_, this, v| Ok({this.z = v;}));

        fields.add_field_method_get("Unit", |_, this| Ok(this.get_unit()));

        fields.add_field_method_get("Magnitude", |_, this| Ok(this.get_magnitude()));
        fields.add_field_method_set("Magnitude", |_, this, val| Ok(this.set_magnitude(val)));
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("Cross",|_, this, other| Ok(this.cross(other)));
        methods.add_method("Abs", |_, this, ()| Ok(this.abs()));
        methods.add_method("Ceil", |_, this, ()| Ok(this.ceil()));
        methods.add_method("Floor", |_, this, ()| Ok(this.floor()));
        methods.add_method("Sign", |_, this, ()| Ok(this.sign()));
        methods.add_method("Angle", 
            |_, this, (other, axis): (Vector3, Option<Vector3>)| 
                Ok(this.get_angle(other, axis))
        );
        methods.add_method("Dot",|_, this, other| Ok(this.dot(other)));
        methods.add_method("Lerp",|_, this, (other, alpha)| Ok(this.lerp(other, alpha)));
        methods.add_method("Max",|_, this, other| Ok(this.max(other)));
        methods.add_method("Min",|_, this, other| Ok(this.min(other)));
        methods.add_method("FuzzyEq",|_, this, other: Vector3 | {
            todo!();
            Ok(())
        });
        methods.add_meta_method("__add", |_, this, other| Ok(*this+other));
        methods.add_meta_method("__sub", |_, this, other| Ok(*this-other));
        methods.add_meta_method("__mul", |lua, this, val: LuaValue| {
            if val.is_number() {
                Ok(*this * unsafe { val.as_number().unwrap_unchecked() })
            } else if val.is_userdata() {
                let result = Vector3::from_lua(val, lua);
                if result.is_ok() {
                    Ok(*this * unsafe { result.unwrap_unchecked() })
                } else {
                    Err(LuaError::BadArgument {
                        to: Some("Vector3::__mul".into()),
                        pos: 2,
                        name: Some("other".into()),
                        cause: Arc::new(result.unwrap_err())
                    })
                }
            } else {
                Err(LuaError::BadArgument {
                    to: Some("Vector3::__mul".into()),
                    pos: 2,
                    name: Some("other".into()),
                    cause: Arc::new(LuaError::FromLuaConversionError {
                        from: val.type_name(),
                        to: "Vector3".into(),
                        message: None
                    })})
            }
        });
        methods.add_meta_method("__div", |lua, this, val: LuaValue| {
            if val.is_number() {
                Ok(*this / unsafe { val.as_number().unwrap_unchecked() })
            } else if val.is_userdata() {
                let result = Vector3::from_lua(val, lua);
                if result.is_ok() {
                    Ok(*this / unsafe { result.unwrap_unchecked() })
                } else {
                    Err(LuaError::BadArgument {
                        to: Some("Vector3::__div".into()),
                        pos: 2,
                        name: Some("other".into()),
                        cause: Arc::new(result.unwrap_err())
                    })
                }
            } else {
                Err(LuaError::BadArgument {
                    to: Some("Vector3::__div".into()),
                    pos: 2,
                    name: Some("other".into()),
                    cause: Arc::new(LuaError::FromLuaConversionError {
                        from: val.type_name(),
                        to: "Vector3".into(),
                        message: None
                    })})
            }
        });
        methods.add_meta_method("__tostring", |_, this, ()| Ok(format!("({}, {}, {})",this.x,this.y,this.z)));
        methods.add_meta_method("__eq", |_, this, other| Ok(*this == other));
    }
}

impl LuaSingleton for Vector3 {
    fn register_singleton(lua: &Lua) -> LuaResult<()> {
        let table = lua.create_table()?;
        table.raw_set(
            "new",
            lua.create_function(|_, (x, y, z): (f64, f64, f64)| {
                Ok(Vector3 {
                    x, y, z
                })
            })?
        )?;
        table.raw_set(
            "FromNormalId",
            lua.create_function(|_, v: NormalId| {
                Ok(Vector3::from(v))
            })?
        )?;
        table.raw_set(
            "FromAxis",
            lua.create_function(|_, v: Axis| {
                Ok(Vector3::from(v))
            })?
        )?;
        table.raw_set("zero", Vector3::ZERO)?;
        table.raw_set("one", Vector3::ONE)?;
        table.raw_set("xAxis", Vector3::X_AXIS)?;
        table.raw_set("yAxis", Vector3::Y_AXIS)?;
        lua.globals().raw_set("Vector3", table)?;
        Ok(())
    }
}

impl LuaUserData for Vector3int16 {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("X", |_, this| Ok(this.x));
        fields.add_field_method_set("X", |_, this, v| Ok({this.x = v;}));
        
        fields.add_field_method_get("Y", |_, this| Ok(this.y));
        fields.add_field_method_set("Y", |_, this, v| Ok({this.y = v;}));
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__add", |_, this, other| Ok(*this+other));
        methods.add_meta_method("__sub", |_, this, other| Ok(*this-other));
        methods.add_meta_method("__mul", |lua, this, val: LuaValue| {
            if val.is_number() {
                Ok(*this * unsafe { val.as_number().unwrap_unchecked() })
            } else if val.is_userdata() {
                let result = Vector3int16::from_lua(val, lua);
                if result.is_ok() {
                    Ok(*this * unsafe { result.unwrap_unchecked() })
                } else {
                    Err(LuaError::BadArgument {
                        to: Some("Vector3int16::__mul".into()),
                        pos: 2,
                        name: Some("other".into()),
                        cause: Arc::new(result.unwrap_err())
                    })
                }
            } else {
                Err(LuaError::BadArgument {
                    to: Some("Vector3int16::__mul".into()),
                    pos: 2,
                    name: Some("other".into()),
                    cause: Arc::new(LuaError::FromLuaConversionError {
                        from: val.type_name(),
                        to: "Vector3int16".into(),
                        message: None
                    })})
            }
        });
        methods.add_meta_method("__div", |lua, this, val: LuaValue| {
            if val.is_number() {
                Ok(*this / unsafe { val.as_number().unwrap_unchecked() })
            } else if val.is_userdata() {
                let result = Vector3int16::from_lua(val, lua);
                if result.is_ok() {
                    Ok(*this / unsafe { result.unwrap_unchecked() })
                } else {
                    Err(LuaError::BadArgument {
                        to: Some("Vector3int16::__div".into()),
                        pos: 2,
                        name: Some("other".into()),
                        cause: Arc::new(result.unwrap_err())
                    })
                }
            } else {
                Err(LuaError::BadArgument {
                    to: Some("Vector3int16::__div".into()),
                    pos: 2,
                    name: Some("other".into()),
                    cause: Arc::new(LuaError::FromLuaConversionError {
                        from: val.type_name(),
                        to: "Vector3int16".into(),
                        message: None
                    })})
            }
        });
        methods.add_meta_method("__tostring", |_, this, ()| Ok(format!("({}, {}, {})",this.x,this.y,this.z)));
    }
}

impl LuaSingleton for Vector3int16 {
    fn register_singleton(lua: &Lua) -> LuaResult<()> {
        let table = lua.create_table()?;
        table.raw_set(
            "new",
            lua.create_function(|_, (x, y, z): (i64, i64, i64)| {
                Ok(Vector3int16 {
                    x: x as i16,
                    y: y as i16,
                    z: z as i16
                })
            })?
        )?;
        lua.globals().raw_set("Vector3int16", table)?;
        Ok(())
    }
}