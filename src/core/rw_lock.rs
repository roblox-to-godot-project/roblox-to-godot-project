use std::mem::MaybeUninit;
use std::sync::RwLock as StdRwLock;
use std::sync::RwLockReadGuard as StdRwLockReadGuard;
use std::sync::RwLockWriteGuard as StdRwLockWriteGuard;

#[derive(Debug, Default)]
pub struct RwLock<T: ?Sized> {
    lock: StdRwLock<()>,
    data: T
}
pub struct RwLockReadGuard<'a, T: ?Sized> {
    guard: StdRwLockReadGuard<'static, ()>,
    lock: &'a RwLock<T>
}
pub struct RwLockWriteGuard<'a, T: ?Sized> {
    guard: StdRwLockWriteGuard<'static, ()>,
    lock: &'a RwLock<T>
}

impl<T> RwLock<T> {
    pub fn new(value: T) -> Self {
        RwLock {
            lock: StdRwLock::default(),
            data: value
        }
    }
    pub fn new_uninit() -> RwLock<MaybeUninit<T>> {
        RwLock {
            lock: StdRwLock::default(),
            data: MaybeUninit::uninit()
        }
    }
    pub fn new_zeroed() -> RwLock<MaybeUninit<T>> {
        RwLock {
            lock: StdRwLock::default(),
            data: MaybeUninit::zeroed()
        }
    }
}
impl<T> RwLock<MaybeUninit<T>> {
    // SAFETY: Make sure the object is initialized.
    pub unsafe fn assume_init(self) -> RwLock<T> {
        RwLock {
            lock: StdRwLock::default(),
            data: self.data.assume_init()
        }
    }
}
impl<T: ?Sized> RwLock<T> {
    //todo!()
}