use std::arch::breakpoint;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::mem::transmute;
use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr::drop_in_place;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use parking_lot::lock_api::RawRwLock as IRawRwLock;
use parking_lot::RawRwLock;

use godot::global::godot_print_rich;

#[cfg(debug_assertions)]
use crate::godot_debug;
#[cfg(debug_assertions)]
use std::{sync::{Mutex, MutexGuard}, thread::{ThreadId, current}, backtrace::Backtrace};

pub struct RwLock<T: ?Sized> {
    lock: RawRwLock,
    poisoned: AtomicBool,
    data: T,
}
impl<T: Debug> Debug for RwLock<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLock")
            //.field("lock", &self.lock)
            .field("poisoned", &self.poisoned)
            .field("data", &self.data).finish()
    }
}

impl<T: Default> Default for RwLock<T> {
    fn default() -> Self {
        Self {
            lock: RawRwLock::INIT,
            poisoned: AtomicBool::new(false),
            data: T::default()
        }
    }
}

pub struct RwLockReadGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>
}
pub struct RwLockWriteGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>
}

impl<'a, T: Debug> Debug for RwLockReadGuard<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockReadGuard").field("lock", &self.lock).finish()
    }
}
impl<'a, T: Debug> Debug for RwLockWriteGuard<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockWriteGuard").field("lock", &self.lock).finish()
    }
}

pub struct RwLockReadReleaseGuard<'a, 'b, T: ?Sized> {
    guard: &'b mut RwLockReadGuard<'a, T>
}
pub struct RwLockWriteReleaseGuard<'a, 'b, T: ?Sized> {
    guard: &'b mut RwLockWriteGuard<'a, T>
}

impl <'a, 'b, T: Debug> Debug for RwLockReadReleaseGuard<'a, 'b, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockReadReleaseGuard").field("guard", &self.guard).finish()
    }
}
impl <'a, 'b, T: Debug> Debug for RwLockWriteReleaseGuard<'a, 'b, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RwLockWriteReleaseGuard").field("guard", &self.guard).finish()
    }
}

pub struct PoisonError<T: ?Sized> {
    guard: T
}
pub type LockResult<T> = Result<T, PoisonError<T>>;
pub type TryLockResult< T> = Result<T, TryLockError<T>>;

unsafe impl<T: ?Sized> Send for RwLock<T> {}
unsafe impl<T: ?Sized> Sync for RwLock<T> {}
pub enum TryLockError<T> {
    WouldBlock,
    Poisoned(PoisonError<T>)
}
impl<T> RwLock<T> {
    pub fn new(value: T) -> Self {
        RwLock {
            lock: RawRwLock::INIT,
            poisoned: AtomicBool::new(false),
            data: value,
        }
    }
    pub fn new_uninit() -> RwLock<MaybeUninit<T>> {
        RwLock {
            lock: RawRwLock::INIT,
            data: MaybeUninit::uninit(),
            poisoned: AtomicBool::new(false)
        }
    }
    pub fn new_zeroed() -> RwLock<MaybeUninit<T>> {
        RwLock {
            lock: RawRwLock::INIT,
            data: MaybeUninit::zeroed(),
            poisoned: AtomicBool::new(false)
        }
    }
    pub fn into_inner(self) -> T {
        self.data
    }
}
impl<T> RwLock<MaybeUninit<T>> {
    // SAFETY: Make sure the object is initialized.
    pub unsafe fn assume_init(self) -> RwLock<T> {
        RwLock {
            lock: RawRwLock::INIT,
            data: self.data.assume_init(),
            poisoned: AtomicBool::new(false)
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    #[inline]
    pub unsafe fn access(&self) -> *mut T {
        (&raw const self.data).cast_mut()
    }
    pub fn read<'a>(&'a self) -> LockResult<RwLockReadGuard<'a, T>> {
        // todo!("make these functions check the poison flag")
        self.lock.lock_shared();
        Ok(RwLockReadGuard {
            lock: self
        })
        }
    pub fn try_read<'a>(&'a self) -> TryLockResult<RwLockReadGuard<'a, T>> {
        if !self.lock.try_lock_shared() {
            Ok(RwLockReadGuard {
                lock: self
            })
        } else {
            Err(TryLockError::WouldBlock)
        }
    }
    pub fn write<'a>(&'a self) -> LockResult<RwLockWriteGuard<'a, T>> {
        self.lock.lock_exclusive();
        Ok(RwLockWriteGuard { lock: self })
        }
    pub fn try_write<'a>(&'a self) -> TryLockResult<RwLockWriteGuard<'a, T>> {
        if self.lock.try_lock_exclusive() {
            Ok(RwLockWriteGuard { lock: self })
        } else {
            Err(TryLockError::WouldBlock)
        }
    }
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Relaxed)
    }
    #[inline]
    pub fn clear_poison(&self) {
        self.poisoned.store(false, Relaxed);
    }
}

impl<'a, T: ?Sized> RwLockReadGuard<'a, T> {
    pub fn guard_release<'b>(&'b mut self) -> RwLockReadReleaseGuard<'a, 'b, T> {
        // SAFETY: Restored when dropping release guard
        unsafe { self.lock.lock.unlock_shared() };
        RwLockReadReleaseGuard { guard: self }
    }
}
impl<'a, T: ?Sized> RwLockWriteGuard<'a, T> {
    pub fn guard_release<'b>(&'b mut self) -> RwLockWriteReleaseGuard<'a, 'b, T> {
        // SAFETY: Restored when dropping release guard
        unsafe { self.lock.lock.unlock_exclusive() };
        RwLockWriteReleaseGuard { guard: self }
    }
}

impl<'a, 'b, T: ?Sized> Drop for RwLockReadReleaseGuard<'a, 'b, T> {
    fn drop(&mut self) {
        self.guard.lock.lock.lock_shared();
    }
}
impl<'a, 'b, T: ?Sized> Drop for RwLockWriteReleaseGuard<'a, 'b, T> {
    fn drop(&mut self) {
        self.guard.lock.lock.lock_exclusive();
    }
}

impl<'a, T: ?Sized> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { self.lock.lock.unlock_shared() };
    }
}
impl<'a, T: ?Sized> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { self.lock.lock.unlock_exclusive() };
    }
}

impl<'a, T: ?Sized> Deref for RwLockReadGuard<'a, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}
impl<'a, T: ?Sized> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}
impl<'a, T: ?Sized> DerefMut for RwLockWriteGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.lock.access().as_mut().unwrap_unchecked() } 
    }
}

impl<'a, T: ?Sized> Borrow<T> for RwLockReadGuard<'a, T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.lock.data
    }
}
impl<'a, T: ?Sized> Borrow<T> for RwLockWriteGuard<'a, T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.lock.data
    }
}
impl<'a, T: ?Sized> BorrowMut<T> for RwLockWriteGuard<'a, T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        unsafe { self.lock.access().as_mut().unwrap_unchecked() } 
    }
}
impl<T> PoisonError<T> {
    #[inline]
    pub fn get_ref(&self) -> &T {
        &self.guard
    }
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.guard
    }
    #[inline]
    pub fn into_inner(self) -> T {
        self.guard
    }
}

impl<T: ?Sized> Debug for PoisonError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PoisonError").finish()
    }
}