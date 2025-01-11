use super::IInstance;
use crate::core::{Irc, IWeak};

pub type ManagedActor = Irc<Actor>;
pub type WeakManagedActor = IWeak<Actor>;

pub struct Actor {}