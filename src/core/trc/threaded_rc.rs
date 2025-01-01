use std::alloc::{alloc, Layout};
use std::borrow::{Borrow, BorrowMut};
use std::mem::{transmute, ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::ptr::{drop_in_place, read, slice_from_raw_parts_mut, write, NonNull};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::core::alloc::{Allocator, Global};
use crate::core::InheritanceBase;

#[inline(always)]
unsafe fn construct_wide_pointer<T>(ptr: *mut u8, layout: Layout) -> NonNull<TrcHeader<[T]>> {
    let ptr: *mut TrcHeader<[T]> = transmute(slice_from_raw_parts_mut(ptr, layout.size()));
    NonNull::new_unchecked(ptr)
}

unsafe fn data_offset<T: ?Sized>(ptr: *const T) -> usize {
    unsafe { data_offset_align(align_of_val(ptr.as_ref().unwrap_unchecked())) }
}

#[inline(always)]
const fn padding_needed_for(layout: &Layout, align: usize) -> usize {
    let align = if align.is_power_of_two() { align } else { usize::MAX };
    let len_rounded_up = unsafe {
        let align_m1 = align.unchecked_sub(1);
        let size_rounded_up = layout.size().unchecked_add(align_m1) & !align_m1;
        size_rounded_up
    };
    unsafe { len_rounded_up.unchecked_sub(layout.size()) }
}

#[inline(always)]
const fn data_offset_align(align: usize) -> usize {
    let layout = Layout::new::<TrcHeader<()>>();
    layout.size() + padding_needed_for(&layout, align)
}

// TODO: Impl clone
#[deprecated]
#[repr(C)]
struct TrcHeader<T>
    where 
        T: ?Sized
{
    pub(self) strong: AtomicU32,
    pub(self) weak: AtomicU32,
    pub(self) lock: RwLock<()>,
    pub(self) data: T
}
fn trc_header_create_layout(layout: Layout) -> Layout {
    Layout::new::<TrcHeader<()>>().extend(layout).unwrap().0.pad_to_align()
}

fn trc_header_drop<T: ?Sized>(header: &mut TrcHeader<T>) {
    todo!()
}

#[deprecated]
pub struct Trc<T: ?Sized, A: Allocator + Send + Sync = Global> {
    header: NonNull<TrcHeader<T>>,
    alloc: A
}
#[deprecated]
pub struct Weak<T: ?Sized, A: Allocator + Send + Sync = Global> {
    header: NonNull<TrcHeader<T>>,
    alloc: A
}
pub struct TrcReadLock<'a, T: ?Sized, A: Allocator + Send + Sync = Global> {
    guard: RwLockReadGuard<'static, ()>,
    rc: &'a Trc<T, A>
}
pub struct TrcWriteLock<'a, T: ?Sized, A: Allocator + Send + Sync = Global> {
    guard: RwLockWriteGuard<'static, ()>,
    rc: &'a Trc<T, A>
}
unsafe impl<T, A: Allocator + Send + Sync> Send for Trc<T, A> {}
unsafe impl<T, A: Allocator + Send + Sync> Sync for Trc<T, A> {}

impl<T: RefUnwindSafe + ?Sized, A: Allocator + Send + Sync + UnwindSafe> UnwindSafe for Trc<T, A> {}
impl<T: RefUnwindSafe + ?Sized, A: Allocator + Send + Sync + UnwindSafe> RefUnwindSafe for Trc<T, A> {}
impl<T: RefUnwindSafe + ?Sized, A: Allocator + Send + Sync + UnwindSafe> UnwindSafe for Weak<T, A> {}
impl<T: RefUnwindSafe + ?Sized, A: Allocator + Send + Sync + UnwindSafe> RefUnwindSafe for Weak<T, A> {}

// Create functions

impl<T> Trc<T> {
    pub fn new(value: T) -> Trc<T> {
        unsafe {
            let mut t= Trc::<T, Global> {
                header: NonNull::new_unchecked(alloc(Layout::new::<TrcHeader<T>>()).cast()),
                alloc: Global
            };
            write(t.header.as_mut(), TrcHeader::<T> {
                strong: AtomicU32::new(1),
                weak: AtomicU32::new(0),
                lock: RwLock::new(()),
                data: value
            });
            t
        }
    }
    pub fn new_cyclic<F>(data_fn: F) -> Trc<T> 
        where F: FnOnce(&Weak<T>) -> T
    {
        unsafe {
            let mut t= Trc::<T, Global> {
                header: NonNull::new_unchecked(alloc(Layout::new::<TrcHeader<T>>()).cast()),
                alloc: Global
            };
            write(&raw mut (t.header.as_mut().strong), AtomicU32::new(1));
            write(&raw mut (t.header.as_mut().weak), AtomicU32::new(0));
            write(&raw mut (t.header.as_mut().lock), RwLock::new(()));
            let weak = t.downgrade();
            write(&raw mut (t.header.as_mut().data), data_fn(&weak));
            t
        }
    }
    pub fn new_uninit() -> Trc<MaybeUninit<T>> {
        unsafe {
            let mut t= Trc::<MaybeUninit<T>, Global> {
                header: NonNull::new_unchecked(alloc(Layout::new::<TrcHeader<MaybeUninit<T>>>()).cast()),
                alloc: Global
            };
            write(t.header.as_mut(), TrcHeader::<MaybeUninit<T>> {
                strong: AtomicU32::new(1),
                weak: AtomicU32::new(0),
                lock: RwLock::new(()),
                data: MaybeUninit::<T>::uninit()
            });
            t
        }
    }
}
impl<T> Default for Trc<T> 
    where T: Default 
{
    fn default() -> Self {
        Trc::<T>::new(T::default())
    }
}
impl<T, A: Allocator + Send + Sync> Trc<T, A> {
    pub fn new_in(value: T, alloc: A) -> Trc<T, A> {
        unsafe {
            let mut t= Trc::<T, A> {
                header: alloc.allocate(Layout::new::<TrcHeader<T>>()).expect("memory allocation failed").cast(),
                alloc
            };
            write(t.header.as_mut(), TrcHeader::<T> {
                strong: AtomicU32::new(1),
                weak: AtomicU32::new(0),
                lock: RwLock::new(()),
                data: value
            });
            t
        }
    }
    pub fn new_cyclic_in<F>(data_fn: F, alloc: A) -> Trc<T, A> 
    where 
        F: FnOnce(&Weak<T, A>) -> T,
        A: Clone
    {
        unsafe {
            let mut t= Trc::<T, A> {
                header: alloc.allocate(Layout::new::<TrcHeader<T>>()).expect("memory allocation failed").cast(),
                alloc
            };
            write(&raw mut (t.header.as_mut().strong), AtomicU32::new(1));
            write(&raw mut (t.header.as_mut().weak), AtomicU32::new(0));
            write(&raw mut (t.header.as_mut().lock), RwLock::new(()));
            let weak = t.downgrade();
            write(&raw mut (t.header.as_mut().data), data_fn(&weak));
            t
        }
    }
    pub fn new_uninit_in(alloc: A) -> Trc<MaybeUninit<T>, A> {
        unsafe {
            let mut t= Trc::<MaybeUninit<T>, A> {
                header: alloc.allocate(Layout::new::<TrcHeader<MaybeUninit<T>>>()).expect("memory allocation failed").cast(),
                alloc
            };
            write(t.header.as_mut(), TrcHeader::<MaybeUninit<T>> {
                strong: AtomicU32::new(1),
                weak: AtomicU32::new(0),
                lock: RwLock::new(()),
                data: MaybeUninit::<T>::uninit()
            });
            t
        }
    }
}
impl<T> Trc<[T]> {
    pub fn new_uninit_slice(count: usize) -> Trc<[MaybeUninit<T>]> {
        unsafe {
            let layout = trc_header_create_layout(Layout::array::<T>(count).expect("failed to create array"));
            
            let mut t= Trc::<[MaybeUninit<T>]> {
                header: construct_wide_pointer(alloc(layout), layout),
                alloc: Global
            };
            write(&raw mut (t.header.as_mut().strong), AtomicU32::new(1));
            write(&raw mut (t.header.as_mut().weak), AtomicU32::new(0));
            write(&raw mut (t.header.as_mut().lock), RwLock::new(()));
            t
        }
    }
}
impl<T, A: Allocator + Send + Sync> Trc<[T], A> {
    pub fn new_uninit_slice_in(count: usize, alloc: A) -> Trc<[MaybeUninit<T>], A> {
        unsafe {
            let layout = trc_header_create_layout(Layout::array::<T>(count).expect("failed to create array"));
            
            let mut t= Trc::<[MaybeUninit<T>], A> {
                header: transmute(alloc.allocate(layout).expect("memory allocation failed")),
                alloc
            };
            write(&raw mut (t.header.as_mut().strong), AtomicU32::new(1));
            write(&raw mut (t.header.as_mut().weak), AtomicU32::new(0));
            write(&raw mut (t.header.as_mut().lock), RwLock::new(()));
            t
        }
    }
}

// Implementations

impl <T: ?Sized, A: Allocator + Send + Sync> Trc<T, A> {
    fn into_inner_with_allocator(self) -> (NonNull<TrcHeader<T>>, A) {
        let this = ManuallyDrop::new(self);
        (this.header, unsafe { read(&this.alloc) })
    }
    unsafe fn from_inner_in(ptr: NonNull<TrcHeader<T>>, alloc: A) -> Trc<T, A> {
        Trc::<T, A> {
            header: ptr,
            alloc
        }
    }
    unsafe fn raw_to_header(ptr: *const T) -> NonNull<TrcHeader<T>> {
        let offset = unsafe { data_offset(ptr) };
        
        unsafe { 
            NonNull::new_unchecked(ptr.byte_sub(offset) as *mut TrcHeader<T>)
        }
    }
    unsafe fn header_to_raw(header: NonNull<TrcHeader<T>>) -> *const T {
        &raw const header.as_ref().data
    }
    pub fn into_raw_with_allocator(self) -> (*const T, A) {
        let (header, alloc) = self.into_inner_with_allocator();
        (unsafe { Self::header_to_raw(header) }, alloc)
    }
    pub unsafe fn from_raw_in(ptr: *const T, alloc: A) -> Trc<T, A> {
        Self::from_inner_in(
            Self::raw_to_header(ptr), 
            alloc
        )
    }
}
/*
impl <T: ?Sized> Trc<T> {
    #[inline(always)]
    pub unsafe fn from_raw(ptr: *const T) -> Trc<T> {
        Trc::from_raw_in(ptr, Global)
    }
}
impl <T: ?Sized, A: Allocator + Send + Sync> Trc<T, A> {
    #[inline]
    pub unsafe fn from_raw_in(ptr: *const T, alloc: A) -> Trc<T, A> {
        Trc {
            header: {
                let layout = trc_header_create_layout(Layout::for_value(ptr.as_ref().unwrap_unchecked()));
                let ptr_mut = ptr.cast_mut();
                let header_size: usize = Layout::new::<TrcHeader<()>>().size();
                let ptr_mut_usize = ptr_mut.byte_offset_from(MaybeUninit::<*const T>::zeroed().assume_init());
                debug_assert!(ptr_mut_usize > 0);
                let ptr_header_usize = ptr_mut_usize as usize - header_size;
                let ptr_header = null_mut::<u8>().byte_offset(ptr_header_usize.try_into().unwrap_unchecked());
                let fat_ptr = slice_from_raw_parts_mut(ptr_header, layout.size());
                let ptr_header = transmute(fat_ptr);
                NonNull::new_unchecked(ptr_header)
            },
            alloc
        }
    }
}
 */
impl<T: ?Sized, A: Allocator + Send + Sync> Trc<T, A> {
    pub unsafe fn as_ptr(&self) -> *const T {
        &raw const *self.access()
    }
    pub fn read(&self) -> TrcReadLock<T, A> {
        unsafe {
            TrcReadLock { 
                guard: transmute::<RwLockReadGuard<'_, ()>, RwLockReadGuard<'static, ()>>(
                    self.header.as_ptr()
                    .as_mut().unwrap_unchecked()
                    .lock.read().expect("object poisoned."),
                ),
                rc: &self
            }
        }
    }
    pub fn write(&self) -> TrcWriteLock<T, A> {
        unsafe {
            TrcWriteLock { 
                guard: transmute::<RwLockWriteGuard<'_, ()>, RwLockWriteGuard<'static, ()>>(
                    self.header.as_ptr()
                    .as_mut().unwrap_unchecked()
                    .lock.write().expect("object poisoned."),
                ),
                rc: &self
            }
        }
    }
    pub unsafe fn access(&self) -> &mut T {
        unsafe {
            &mut self.header.as_ptr()
                .as_mut().unwrap_unchecked()
                .data
        }
    }
    pub fn downgrade(&self) -> Weak<T, A>
    where
        A: Clone
    {
        unsafe {
            self.header.as_ptr().as_mut().unwrap_unchecked()
                .weak.fetch_add(1, Ordering::Relaxed);
        }
        Weak {
            header: self.header,
            alloc: self.alloc.clone()
        }
    }
}

impl<T, A: Allocator + Send + Sync> Trc<MaybeUninit<T>, A> {
    pub unsafe fn assume_init(self) -> Trc<T, A> {
        unsafe {
            let (header_ptr, alloc) = self.into_inner_with_allocator();
            let mut header_ptr: NonNull<TrcHeader<T>> = header_ptr.cast();
            let header = header_ptr.as_mut();
            assert_eq!(header.strong.load(Ordering::Relaxed), 1, "Expected to be the only holder of this object.");
            assert_eq!(header.weak.load(Ordering::Relaxed), 0, "Expect to be the only holder of this object (weak references also count).");
            Trc::<T, A> {
                header: header_ptr,
                alloc
            }
            // self is consumed but not dropped. ref count remains 1
        }
    }
}
impl<T, A: Allocator + Send + Sync> Trc<[MaybeUninit<T>], A> { 
    pub unsafe fn assume_init(self) -> Trc<[T], A> {
        unsafe {
            let (header_ptr, alloc) = self.into_inner_with_allocator();
            let mut header_ptr: NonNull<TrcHeader<[T]>> = transmute(header_ptr);
            let header = header_ptr.as_mut();
            assert_eq!(header.strong.load(Ordering::Relaxed), 1, "Expected to be the only holder of this object.");
            assert_eq!(header.weak.load(Ordering::Relaxed), 0, "Expect to be the only holder of this object (weak references also count).");
            Trc::<[T], A> {
                header: header_ptr,
                alloc
            }
            // self is consumed, ref count remains 1
        }
    }
}

impl<T, A> Weak<T, A>
where 
    T: ?Sized,
    A: Allocator + Send + Sync + Clone
{
    pub fn upgrade(&self) -> Option<Trc<T, A>> {
        unsafe {
            let strong = &mut self.header.clone().as_mut().strong;
            if strong.load(Ordering::Relaxed) == 0 {
                None
            } else {
                strong.fetch_add(1, Ordering::Relaxed);
                Some(Trc::<T, A> {
                    header: self.header,
                    alloc: self.alloc.clone()
                })
            }
        }
    }
}

impl<'a, T: ?Sized, A: Allocator + Send + Sync> Deref for TrcReadLock<'a, T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {&*self.rc.as_ptr()}
    }
}
impl<'a, T: ?Sized> Deref for TrcWriteLock<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {&*self.rc.as_ptr()}
    }
}
impl<'a, T: ?Sized> DerefMut for TrcWriteLock<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {self.rc.access()}
    }
}

impl<'a, T: ?Sized> Borrow<T> for TrcReadLock<'a, T> {
    fn borrow(&self) -> &T {
        unsafe {&*self.rc.as_ptr()}
    }
}
impl<'a, T: ?Sized> Borrow<T> for TrcWriteLock<'a, T> {
    fn borrow(&self) -> &T {
        unsafe {&*self.rc.as_ptr()}
    }
}
impl<'a, T: ?Sized> BorrowMut<T> for TrcWriteLock<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        unsafe {self.rc.access()}
    }
}

// Downcasting and upcasting maybe
/*
impl<T: InheritanceBase, A: Allocator + Send + Sync> Trc<T, A> {
    pub fn downcast<U: InheritanceBase + 'static>(self) -> Result<Trc<U, A>, Trc<T, A>> {
        unsafe {
            if self.header.as_ref().strong.load(Ordering::Relaxed) != 1 || self.header.as_ref().weak.load(Ordering::Relaxed) != 0 {
                return Err(self);
            }
            //SAFETY: We're the only one which has access to it.
            let base = self.access() as &mut dyn InheritanceBase;
            base.
        }
    }
} */

impl<'a, T: ?Sized, A: Allocator + Send + Sync> Drop for Trc<T, A> {
    fn drop(&mut self) {
        unsafe {
            let header = self.header.as_mut();
            if header.strong.fetch_sub(1, Ordering::Relaxed) == 1 { // this was the last one
                drop_in_place(&raw mut header.data);
                if header.weak.load(Ordering::Relaxed) == 0 {
                    trc_header_drop(header);
                }
            }
        }
    }
}

impl<'a, T: ?Sized, A: Allocator + Send + Sync> Drop for Weak<T, A> {
    fn drop(&mut self) {
        unsafe {
            let header = self.header.as_mut();
            if header.weak.fetch_sub(1, Ordering::Relaxed) == 1 { // this was the last one
                if header.strong.load(Ordering::Relaxed) == 0 {
                    trc_header_drop(header);
                }
            }
        }
    }
}