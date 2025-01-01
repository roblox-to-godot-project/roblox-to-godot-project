use super::IInstance;
use crate::core::{ITrc, IWeak};

pub type ManagedActor = ITrc<Actor>;
pub type WeakManagedActor = IWeak<Actor>;

pub struct Actor {}