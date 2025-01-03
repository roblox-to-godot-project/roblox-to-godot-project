use std::alloc::Layout;
use std::borrow::{Borrow, BorrowMut};
use std::mem::{transmute, ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::TryLockError as RwTryLockError;

use crate::core::{fat_to_metadata, inheritance_cast_to_mut, inheritance_is_of_type, thin_to_fat_mut, PtrMetadata};
use crate::core::{alloc::{Allocator, Global}, null_mut, InheritanceBase};
use crate::godot_debug;
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct PoisonError;
pub type LockResult<T> = Result<T, PoisonError>;
pub type TryLockResult<T> = Result<T, TryLockError>;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TryLockError {
    Poisoned,
    WouldBlock
}

fn create_layout_for_header(layout: Layout) -> Layout {
    Layout::new::<ITrcHead<()>>().extend(layout).unwrap().0
}

#[repr(C)]
pub struct ITrcHead<T: ?Sized> {
    layout: Layout, // represents T's size
    destroy: unsafe fn(*mut u8) -> (),
    strong: AtomicU32,
    weak: AtomicU32,
    lock: RwLock<()>,
    base: *mut dyn InheritanceBase,
    data: ManuallyDrop<T>
}

impl<T: Sized + InheritanceBase> ITrcHead<T> {
    fn new<A: Allocator>(value: T, alloc: &A) -> NonNull<Self> {
        let ptr = alloc.allocate(Layout::new::<Self>()).unwrap().cast();
        unsafe {
            ptr.write(ITrcHead::<T> {
                layout: Layout::new::<T>(),
                destroy: |data| {
                    let t: *mut T = data as *mut T;
                    t.drop_in_place();
                },
                strong: AtomicU32::new(1),
                weak: AtomicU32::new(1),
                lock: RwLock::new(()),
                base: null_mut(),
                data: ManuallyDrop::new(value)
            });
            ptr.as_ptr().as_mut().unwrap_unchecked().base = (&raw mut ptr.as_ptr().as_mut().unwrap_unchecked().data) as *mut T as *mut dyn InheritanceBase;
        }
        ptr
    }
    fn new_cyclic<A, F>(data_fn: F, alloc: &A) -> NonNull<Self> 
    where 
        F: FnOnce (&IWeak<T, A>) -> T,
        A: Allocator + Send + Sync + Clone
    {
        let mut ptr = alloc.allocate(Layout::new::<Self>()).unwrap().cast();
        let data_ptr;
        unsafe {
            ptr.write(ITrcHead::<T> {
                layout: Layout::new::<T>(),
                destroy: |data| {
                    let t: *mut T = data as *mut T;
                    t.drop_in_place();
                },
                strong: AtomicU32::new(1),
                weak: AtomicU32::new(2),
                lock: RwLock::new(()),
                base: null_mut(),
                data: ManuallyDrop::new(MaybeUninit::uninit().assume_init())
            });
            ptr.as_ptr().as_mut().unwrap_unchecked().base = (&raw mut ptr.as_ptr().as_mut().unwrap_unchecked().data) as *mut T as *mut dyn InheritanceBase;
            
            data_ptr = (&raw mut ptr.as_mut().data).cast();
        }
        let weak = IWeak::<T, A> {
            head: ptr,
            ptr: data_ptr,
            alloc: alloc.clone()
        };
        unsafe {
            data_ptr.write(data_fn(&weak));
        }
        ptr
    }
    fn new_uninit<A: Allocator>(alloc: &A) -> NonNull<ITrcHead<MaybeUninit<T>>> {
        let ptr = alloc.allocate(Layout::new::<ITrcHead<MaybeUninit<T>>>()).unwrap().cast();
        unsafe {
            ptr.write(ITrcHead::<MaybeUninit<T>> {
                layout: Layout::new::<T>(),
                destroy: |data| {
                    let t: *mut T = data as *mut T;
                    t.drop_in_place();
                },
                strong: AtomicU32::new(1),
                weak: AtomicU32::new(1),
                lock: RwLock::new(()),
                base: null_mut(),
                data: ManuallyDrop::new(MaybeUninit::uninit())
            });
        }
        ptr
    }
}

impl<T: ?Sized> ITrcHead<T> {
    unsafe fn drop_in_place(&mut self) {
        (self.destroy)((&raw mut self.data).cast())
    }
}
#[derive(Debug)]
pub struct ITrc<T, A = Global>
where
    T: ?Sized,
    A: Allocator + Send + Sync
{
    head: NonNull<ITrcHead<T>>,
    ptr: *mut T,
    alloc: A
}


impl<T: RefUnwindSafe + ?Sized, A: Allocator + Send + Sync + UnwindSafe> UnwindSafe for ITrc<T, A> {}
impl<T: RefUnwindSafe + ?Sized, A: Allocator + Send + Sync + UnwindSafe> RefUnwindSafe for ITrc<T, A> {}
impl<T: RefUnwindSafe + ?Sized, A: Allocator + Send + Sync + UnwindSafe> UnwindSafe for IWeak<T, A> {}
impl<T: RefUnwindSafe + ?Sized, A: Allocator + Send + Sync + UnwindSafe> RefUnwindSafe for IWeak<T, A> {}

impl<T, A> ITrc<T, A>
where
    T: ?Sized,
    A: Allocator + Send + Sync
{
    fn deconstruct(self) -> (NonNull<ITrcHead<T>>, *mut T, A) {
        let t = ManuallyDrop::new(self);
        (t.head, t.ptr, unsafe { (&raw const t.alloc).read() })
    }
    
    pub unsafe fn increment_strong_count(&self) {
        if self.head.as_ref().strong.fetch_add(1, Ordering::Acquire) == 0 {
            self.head.as_ref().weak.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub unsafe fn decrement_strong_count(&self) -> (bool, bool) {
        if self.head.as_ref().strong.fetch_sub(1, Ordering::Acquire) == 1 {
            (true, self.head.as_ref().weak.fetch_sub(1, Ordering::Relaxed) == 1)
        } else {
            (false, false)
        }
    }
    unsafe fn increment_strong_count_if_exists(&self) -> bool {
        self.head.as_ref().strong.fetch_update(Ordering::Release, Ordering::Relaxed, |x| {
            if x == 0 {
                None
            } else {
                Some(x+1)
            }
        }).is_ok()
    }
    pub unsafe fn increment_weak_count(&self) {
        self.head.as_ref().weak.fetch_add(1, Ordering::Relaxed);
    }
    pub unsafe fn decrement_weak_count(&self) -> bool {
        self.head.as_ref().weak.fetch_sub(1, Ordering::Relaxed) == 1
    }
    pub fn strong_count(&self) -> u32 {
        unsafe { self.head.as_ref().strong.load(Ordering::Relaxed) }
    }
    pub fn weak_count(&self) -> u32 {
        unsafe { self.head.as_ref().weak.load(Ordering::Relaxed) }
    }
}

impl<T, A> IWeak<T, A>
where
    T: ?Sized,
    A: Allocator + Send + Sync
{
    fn deconstruct(self) -> (NonNull<ITrcHead<T>>, *mut T, A) {
        let t = ManuallyDrop::new(self);
        (t.head, t.ptr, unsafe { (&raw const t.alloc).read() })
    }
    
    unsafe fn increment_strong_count(&self) {
        if self.head.as_ref().strong.fetch_add(1, Ordering::Acquire) == 0 {
            self.head.as_ref().weak.fetch_add(1, Ordering::Relaxed);
        }
    }
    unsafe fn decrement_strong_count(&self) -> (bool, bool) {
        if self.head.as_ref().strong.fetch_sub(1, Ordering::Acquire) == 1 {
            (true, self.head.as_ref().weak.fetch_sub(1, Ordering::Relaxed) == 1)
        } else {
            (false, false)
        }
    }
    unsafe fn increment_strong_count_if_exists(&self) -> bool {
        self.head.as_ref().strong.fetch_update(Ordering::Release, Ordering::Relaxed, |x| {
            if x == 0 {
                None
            } else {
                Some(x+1)
            }
        }).is_ok()
    }
    unsafe fn increment_weak_count(&self) {
        self.head.as_ref().weak.fetch_add(1, Ordering::Relaxed);
    }
    unsafe fn decrement_weak_count(&self) -> bool {
        self.head.as_ref().weak.fetch_sub(1, Ordering::Relaxed) == 1
    }
    fn strong_count(&self) -> u32 {
        unsafe { self.head.as_ref().strong.load(Ordering::Relaxed) }
    }
    fn weak_count(&self) -> u32 {
        unsafe { self.head.as_ref().weak.load(Ordering::Relaxed) }
    }
}

impl<T> ITrc<T>
where
    T: Sized + InheritanceBase,
{
    pub fn new(value: T) -> Self {
        let head = ITrcHead::<T>::new(value, &Global);
        ITrc::<T> {
            head,
            ptr: unsafe { &raw const head.as_ref().data }.cast_mut().cast(),
            alloc: Global
        }
    }
    pub fn new_cyclic<F>(data_fn: F) -> Self
    where
        F: FnOnce(&IWeak<T>) -> T
    {
        let head = ITrcHead::<T>::new_cyclic(data_fn, &Global);
        ITrc::<T> {
            head,
            ptr: unsafe { &raw const head.as_ref().data }.cast_mut().cast(),
            alloc: Global
        }
    }
    pub fn new_uninit() -> ITrc<MaybeUninit<T>> {
        let head = ITrcHead::<T>::new_uninit( &Global);
        ITrc::<MaybeUninit<T>> {
            head,
            ptr: unsafe { &raw const head.as_ref().data }.cast_mut().cast(),
            alloc: Global
        }
    }
}

impl<T, A> ITrc<T, A>
where
    T: Sized + InheritanceBase,
    A: Allocator + Send + Sync
{
    pub fn new_in(value: T, alloc: A) -> Self {
        let head = ITrcHead::<T>::new(value, &alloc);
        ITrc::<T, A> {
            head,
            ptr: unsafe { &raw const head.as_ref().data }.cast_mut().cast(),
            alloc
        }
    }
    pub fn new_cyclic_in<F>(data_fn: F, alloc: A) -> Self
    where
        F: FnOnce(&IWeak<T, A>) -> T,
        A: Clone
    {
        let head = ITrcHead::<T>::new_cyclic(data_fn, &alloc);
        ITrc::<T, A> {
            head,
            ptr: unsafe { &raw const head.as_ref().data }.cast_mut().cast(),
            alloc
        }
    }
    pub fn new_uninit_in(alloc: A) -> ITrc<MaybeUninit<T>, A> {
        let head = ITrcHead::<T>::new_uninit(&alloc);
        ITrc::<MaybeUninit<T>, A> {
            head,
            ptr: unsafe { &raw const head.as_ref().data }.cast_mut().cast(),
            alloc
        }
    }
}


impl<T, A> ITrc<T, A>
where
    T: InheritanceBase + ?Sized + 'static,
    A: Allocator + Send + Sync
{
    pub fn cast_unsized<U: 'static + ?Sized>(self) -> Result<ITrc<U, A>, ITrc<T, A>> {
        let (head, ptr, alloc) = self.deconstruct();
        let p = head.as_ptr();
        let metadata = unsafe { fat_to_metadata(p) }; 
        let base =  unsafe { head.as_ptr().as_mut().unwrap_unchecked().base.as_mut().unwrap_unchecked() };
        let result = inheritance_cast_to_mut!(base, U);
        if result.is_ok() {
            unsafe {
                let new_ptr = &raw mut *result.unwrap_unchecked();
                Ok(ITrc::<U, A> {
                    head: NonNull::new_unchecked(thin_to_fat_mut(head.cast().as_ptr(), metadata)),
                    ptr: new_ptr,
                    alloc
                })
            }
        } else {
            Err(ITrc::<T, A> {
                head,
                ptr,
                alloc
            })
        }
    }
    pub fn is<U: 'static + ?Sized>(&self) -> bool {
        let base =  unsafe { self.head.as_ref().base.as_ref().unwrap_unchecked() };
        inheritance_is_of_type!(base, U)
    }
}
impl<T, A> ITrc<T, A>
where
    T: InheritanceBase + 'static,
    A: Allocator + Send + Sync
{
    pub fn cast_sized<U: 'static + ?Sized>(self) -> Result<ITrc<U, A>, ITrc<T, A>> {
        let (head, ptr, alloc) = self.deconstruct();
        let p = head.as_ptr();
        let metadata = unsafe { fat_to_metadata(p as *mut ITrcHead<dyn InheritanceBase>) }; 
        let base =  unsafe { head.as_ptr().as_mut().unwrap_unchecked().base.as_mut().unwrap_unchecked() };
        let result = inheritance_cast_to_mut!(base, U);
        if result.is_ok() {
            unsafe {
                let new_ptr = &raw mut *result.unwrap_unchecked();
                Ok(ITrc::<U, A> {
                    head: NonNull::new_unchecked(thin_to_fat_mut(head.cast().as_ptr(), metadata)),
                    ptr: new_ptr,
                    alloc
                })
            }
        } else {
            Err(ITrc::<T, A> {
                head,
                ptr,
                alloc
            })
        }
    }
}

impl<T, A> ITrc<MaybeUninit<T>, A>
where
    T: InheritanceBase,
    A: Allocator + Send + Sync
{
    pub unsafe fn assume_init(self) -> ITrc<T, A> {
        let (head, ptr, alloc) = self.deconstruct();
        let ptr = ptr.cast();
        head.as_ptr().as_mut().unwrap_unchecked().base = ptr as *mut dyn InheritanceBase;
        ITrc::<T, A> {
            head: head.cast(),
            ptr,
            alloc
        }
    }
}

impl<T, A> ITrc<T, A>
where
    T: ?Sized,
    A: Allocator + Send + Sync
{
    pub unsafe fn access(&self) -> *mut T {
        self.ptr
    }
    pub fn read<'a>(&'a self) -> LockResult<ITrcReadLock<'a, T, A>> {
        let lock = unsafe { self.head.as_ptr().as_mut().unwrap_unchecked().lock.read() };
        if lock.is_ok() {
            unsafe { Ok(ITrcReadLock { 
                read: transmute(lock.unwrap_unchecked()), // SAFETY: Casting away lifetime, putting it in rc field
                rc: &self
            })}
        } else {
            Err(PoisonError)
        }
    }
    pub fn write<'a>(&'a self) -> LockResult<ITrcWriteLock<'a, T, A>> {
        let lock = unsafe { self.head.as_ptr().as_mut().unwrap_unchecked().lock.write() };
        if lock.is_ok() {
            unsafe { Ok(ITrcWriteLock { 
                write: transmute(lock.unwrap_unchecked()), // SAFETY: Casting away lifetime, putting it in rc field
                rc: &self
            })}
        } else {
            Err(PoisonError)
        }
    }
    pub fn is_poisoned(&self) -> bool {
        unsafe {
            self.head.as_ptr().as_mut().unwrap_unchecked().lock.is_poisoned()
        }
    }
    pub fn clear_poison(&self) {
        unsafe {
            self.head.as_ptr().as_mut().unwrap_unchecked().lock.clear_poison();
        }
    }
    pub fn try_read<'a>(&'a self) -> TryLockResult<ITrcReadLock<'a, T, A>> {
        let lock = unsafe { self.head.as_ptr().as_mut().unwrap_unchecked().lock.try_read() };
        if lock.is_ok() {
            unsafe { Ok(ITrcReadLock { 
                read: transmute(lock.unwrap_unchecked()), // SAFETY: Casting away lifetime, putting it in rc field
                rc: &self
            })}
        } else {
            unsafe {
                Err(match lock.unwrap_err_unchecked() {
                    RwTryLockError::Poisoned(_) => TryLockError::Poisoned,
                    RwTryLockError::WouldBlock => TryLockError::WouldBlock
                })
            }
        }
    }
    pub fn try_write<'a>(&'a self) -> TryLockResult<ITrcWriteLock<'a, T, A>> {
        let lock = unsafe { self.head.as_ptr().as_mut().unwrap_unchecked().lock.try_write() };
        if lock.is_ok() {
            unsafe { Ok(ITrcWriteLock { 
                write: transmute(lock.unwrap_unchecked()), // SAFETY: Casting away lifetime, putting it in rc field
                rc: &self
            })}
        } else {
            unsafe {
                Err(match lock.unwrap_err_unchecked() {
                    RwTryLockError::Poisoned(_) => TryLockError::Poisoned,
                    RwTryLockError::WouldBlock => TryLockError::WouldBlock
                })
            }
        }
    }
}

impl<T, A> ITrc<T, A>
where
    T: ?Sized,
    A: Allocator + Send + Sync + Clone
{
    pub fn downgrade(&self) -> IWeak<T, A> {
        unsafe { self.increment_weak_count(); }
        IWeak { head: self.head, ptr: self.ptr, alloc: self.alloc.clone() }
    }
}

#[derive(Debug)]
pub struct IWeak<T, A = Global>
where
    T: ?Sized,
    A: Allocator + Send + Sync
{
    head: NonNull<ITrcHead<T>>,
    ptr: *mut T,
    alloc: A
}

pub struct ITrcReadLock<'a, T, A = Global>
where
    T: ?Sized,
    A: Allocator + Send + Sync
{
    read: RwLockReadGuard<'static, ()>,
    rc: &'a ITrc<T, A>
}

pub struct ITrcWriteLock<'a, T, A = Global>
where
    T: ?Sized,
    A: Allocator + Send + Sync
{
    write: RwLockWriteGuard<'static, ()>,
    rc: &'a ITrc<T, A>
}

unsafe impl<T, A> Send for ITrc<T, A> where
    T: ?Sized,
    A: Allocator + Send + Sync
{}
unsafe impl<T, A> Sync for ITrc<T, A> where
    T: ?Sized,
    A: Allocator + Send + Sync
{}

unsafe impl<T, A> Send for IWeak<T, A> where
    T: ?Sized,
    A: Allocator + Send + Sync
{}
unsafe impl<T, A> Sync for IWeak<T, A> where
    T: ?Sized,
    A: Allocator + Send + Sync
{}


impl<T, A> Drop for ITrc<T, A>
where
    T: ?Sized,
    A: Allocator + Send + Sync
{
    #[inline]
    fn drop(&mut self) {
        let head = unsafe {
            self.head.as_mut()
        };
        match unsafe {self.decrement_strong_count()}  {
            (true, false) => unsafe { head.drop_in_place(); },
            (true, true) => unsafe {
                head.drop_in_place();
                self.alloc.deallocate(
                    self.head.cast(),
                    create_layout_for_header(head.layout)
                );
            },
            _ => ()
        }
    }
}

impl<'a, T: ?Sized, A: Allocator + Send + Sync> Deref for ITrcReadLock<'a, T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {&*self.rc.access()}
    }
}
impl<'a, T: ?Sized> Deref for ITrcWriteLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {&*self.rc.access()}
    }
}
impl<'a, T: ?Sized> DerefMut for ITrcWriteLock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {&mut *self.rc.access()}
    }
}

impl<'a, T: ?Sized> Borrow<T> for ITrcReadLock<'a, T> {
    fn borrow(&self) -> &T {
        unsafe {&*self.rc.access()}
    }
}
impl<'a, T: ?Sized> Borrow<T> for ITrcWriteLock<'a, T> {
    fn borrow(&self) -> &T {
        unsafe {&*self.rc.access()}
    }
}
impl<'a, T: ?Sized> BorrowMut<T> for ITrcWriteLock<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        unsafe {&mut *self.rc.access()}
    }
}

impl<T, A> Drop for IWeak<T, A>
where
    T: ?Sized,
    A: Allocator + Send + Sync
{
    #[inline]
    fn drop(&mut self) {
        let head = unsafe { self.head.as_mut() };
        if unsafe { self.decrement_weak_count() } {
            unsafe { self.alloc.deallocate(
                self.head.cast(),
                create_layout_for_header(head.layout)
            ) };
        }
    }
}

impl<T, A> IWeak<T, A>
where
    T: ?Sized,
    A: Allocator + Send + Sync + Clone
{
    pub fn upgrade(&self) -> Option<ITrc<T, A>> {
        if unsafe { self.increment_strong_count_if_exists() } {
            Some(ITrc::<T, A> {
                head: self.head,
                ptr: self.ptr,
                alloc: self.alloc.clone()
            })
        } else {
            None
        }
    }
    pub fn dead(&self) -> bool {
        self.weak_count() == 0
    }
    pub fn into_inner_with_allocator(self) -> (NonNull<ITrcHead<T>>, *mut T, A) {
        let m = ManuallyDrop::new(self);
        let a = unsafe { (&raw const m.alloc).read() };
        let p = m.ptr;
        let head = m.head;
        (head, p, a)
    }
    pub fn from_inner_with_allocator(tuple: (NonNull<ITrcHead<T>>, *mut T, A)) -> Self {
        IWeak {
            head: tuple.0,
            ptr: tuple.1,
            alloc: tuple.2
        }
    }
}

impl <T, A> Clone for ITrc<T, A>
where
    T: ?Sized,
    A: Allocator + Send + Sync + Clone
{
    fn clone(&self) -> Self {
        unsafe { self.increment_strong_count(); }
        Self {
            head: self.head,
            ptr: self.ptr,
            alloc: self.alloc.clone()
        }
    }
}
impl <T, A> Clone for IWeak<T, A>
where
    T: ?Sized,
    A: Allocator + Send + Sync + Clone
{
    fn clone(&self) -> Self {
        unsafe { self.increment_weak_count(); }
        Self {
            head: self.head,
            ptr: self.ptr,
            alloc: self.alloc.clone()
        }
    }
}