use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::mem::transmute;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::RwLock as StdRwLock;
use std::sync::RwLockReadGuard as StdRwLockReadGuard;
use std::sync::RwLockWriteGuard as StdRwLockWriteGuard;
use std::sync::TryLockError as StdTryLockError;

#[derive(Debug, Default)]
pub struct RwLock<T: ?Sized> {
    lock: StdRwLock<()>,
    data: T
}
#[derive(Debug)]
pub struct RwLockReadGuard<'a, T: ?Sized> {
    guard: StdRwLockReadGuard<'static, ()>,
    lock: &'a RwLock<T>
}
#[derive(Debug)]
pub struct RwLockWriteGuard<'a, T: ?Sized> {
    guard: StdRwLockWriteGuard<'static, ()>,
    lock: &'a RwLock<T>
}
#[derive(Debug)]
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
    pub fn into_inner(self) -> T {
        self.data
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
    #[inline]
    pub unsafe fn access(&self) -> *mut T {
        (&raw const self.data).cast_mut()
    }
    pub fn read<'a>(&'a self) -> LockResult<RwLockReadGuard<'a, T>> {
        self.lock.read()
            .map(|guard| RwLockReadGuard {
                guard: unsafe { transmute(guard) }, 
                lock: self
            })
            .map_err(|guard| PoisonError {
                guard: RwLockReadGuard {
                    guard: unsafe { transmute(guard.into_inner()) },
                    lock: self
            }})
        }
    pub fn try_read<'a>(&'a self) -> TryLockResult<RwLockReadGuard<'a, T>> {
        self.lock.try_read()
            .map(|guard| RwLockReadGuard {
                guard: unsafe { transmute(guard) }, 
                lock: self
            })
            .map_err(|error| match error {
                StdTryLockError::Poisoned(guard) => 
                    TryLockError::Poisoned(PoisonError {
                        guard: RwLockReadGuard {
                            guard: unsafe { transmute(guard.into_inner()) },
                            lock: self
                }}),
                StdTryLockError::WouldBlock => TryLockError::WouldBlock
            } )
    }
    pub fn write<'a>(&'a self) -> LockResult<RwLockWriteGuard<'a, T>> {
        self.lock.write()
            .map(|guard| RwLockWriteGuard {
                guard: unsafe { transmute(guard) }, 
                lock: self
            })
            .map_err(|guard| PoisonError {
                guard: RwLockWriteGuard {
                    guard: unsafe { transmute(guard.into_inner()) },
                    lock: self
            }})
        }
    pub fn try_write<'a>(&'a self) -> TryLockResult<RwLockWriteGuard<'a, T>> {
        self.lock.try_write()
            .map(|guard| RwLockWriteGuard {
                guard: unsafe { transmute(guard) }, 
                lock: self
            })
            .map_err(|error| match error {
                StdTryLockError::Poisoned(guard) => 
                    TryLockError::Poisoned(PoisonError {
                        guard: RwLockWriteGuard {
                            guard: unsafe { transmute(guard.into_inner()) },
                            lock: self
                }}),
                StdTryLockError::WouldBlock => TryLockError::WouldBlock
            } )
    }
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.lock.is_poisoned()
    }
    #[inline]
    pub fn clear_poison(&self) {
        self.lock.clear_poison();
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