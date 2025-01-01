pub mod alloc;
mod trc;
mod security;
mod state;
mod scheduler;
mod vm;
mod inheritance;
mod pointers;
mod instance_repl_table;
mod rw_lock;

pub(self) use instance_repl_table::InstanceReplicationTable;
pub use inheritance::*;
pub use trc::*;
pub use vm::RobloxVM;
pub use state::{LuauState, registry_keys};
pub use security::*;
pub(self) use pointers::*;