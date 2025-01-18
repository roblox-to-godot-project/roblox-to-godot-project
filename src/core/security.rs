use std::ops::BitOr;
#[derive(Eq, PartialEq, Clone, Copy, PartialOrd, Ord)]
pub struct SecurityContext(u8);
impl SecurityContext {
    pub const NONE: SecurityContext = SecurityContext(0);
    pub const PLUGIN: SecurityContext = SecurityContext(0x1);
    pub const ROBLOX_PLACE: SecurityContext = SecurityContext(0x2);
    pub const WRITE_PLAYER: SecurityContext = SecurityContext(0x4);
    pub const LOCAL_USER: SecurityContext = SecurityContext(0x10);
    pub const ROBLOX_SCRIPT: SecurityContext = SecurityContext(0x1D);
    pub const ROBLOX: SecurityContext = SecurityContext(0x1F);
    pub const TEST_LOCAL_USER: SecurityContext = SecurityContext(0x10);
}

impl BitOr for SecurityContext {
    type Output = SecurityContext;
    fn bitor(self, rhs: Self) -> Self::Output {
        SecurityContext(self.0 | rhs.0)
    }
}

impl Into<u8> for SecurityContext {
    fn into(self) -> u8 {
        self.0
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThreadIdentityType {
    ANON,
    USERINIT,
    SCRIPT,
    SCRIPTINROBLOXPLACE,
    SCRIPTBYROBLOX,
    STUDIOCOMMANDBAR,
    STUDIOPLUGIN,
    WEBSERV,
    REPL
}
#[inline]
pub fn get_security_context_for_identity(t: ThreadIdentityType) -> SecurityContext {
    match t {
        ThreadIdentityType::ANON => SecurityContext::NONE,
        ThreadIdentityType::USERINIT => SecurityContext::PLUGIN | SecurityContext::ROBLOX_PLACE | SecurityContext::LOCAL_USER,
        ThreadIdentityType::SCRIPT => SecurityContext::NONE,
        ThreadIdentityType::SCRIPTINROBLOXPLACE => SecurityContext::ROBLOX_PLACE,
        ThreadIdentityType::SCRIPTBYROBLOX => SecurityContext::PLUGIN | SecurityContext::ROBLOX_PLACE | SecurityContext::LOCAL_USER | SecurityContext::ROBLOX_SCRIPT,
        ThreadIdentityType::STUDIOCOMMANDBAR => SecurityContext::PLUGIN | SecurityContext::ROBLOX_PLACE | SecurityContext::LOCAL_USER,
        ThreadIdentityType::STUDIOPLUGIN => SecurityContext::PLUGIN,
        ThreadIdentityType::WEBSERV => SecurityContext::ROBLOX,
        ThreadIdentityType::REPL => SecurityContext::WRITE_PLAYER | SecurityContext::ROBLOX_PLACE | SecurityContext::ROBLOX_SCRIPT
    }
}