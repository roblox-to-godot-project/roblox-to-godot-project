use std::{alloc::Layout, mem::ManuallyDrop, ptr::{slice_from_raw_parts, slice_from_raw_parts_mut}};

#[inline(always)]
unsafe fn transmute_unchecked<A, B>(src: A) -> B {
    debug_assert!(Layout::new::<A>() == Layout::new::<B>());
    transmute_unchecked_unsized(src)
}
#[inline(always)]
const unsafe fn transmute_unchecked_unsized<A, B>(src: A) -> B {
    #[repr(C)]
    union TransmuteInlined<A, B> {
        p1: ManuallyDrop<A>,
        p2: ManuallyDrop<B>
    }
    let f = TransmuteInlined::<A, B> {
        p1: ManuallyDrop::new(src)
    };
    ManuallyDrop::into_inner(f.p2)
}
#[inline(always)]
pub unsafe fn fat_to_thin<T: ?Sized>(ptr: *const T) -> *const u8 {
    transmute_unchecked::<*const T, (*const u8, usize)>(ptr).0
}
#[inline(always)]
pub unsafe fn fat_to_thin_mut<T: ?Sized>(ptr: *mut T) -> *mut u8 {
    transmute_unchecked::<*mut T, (*mut u8, usize)>(ptr).0
}
#[inline(always)]
pub unsafe fn thin_to_fat_trait<T: ?Sized>(ptr: *const u8, vtable: *const u8) -> *const T {
    transmute_unchecked((ptr, vtable))
}
#[inline(always)]
pub unsafe fn thin_to_fat<T: ?Sized>(ptr: *const u8, size: usize) -> *const T {
    transmute_unchecked(slice_from_raw_parts(ptr, size))
}
#[inline(always)]
pub unsafe fn thin_to_fat_trait_mut<T: ?Sized>(ptr: *mut u8, vtable: *mut u8) -> *mut T {
    transmute_unchecked((ptr, vtable))
}
#[inline(always)]
pub unsafe fn thin_to_fat_mut<T: ?Sized>(ptr: *mut u8, size: usize) -> *mut T {
    transmute_unchecked(slice_from_raw_parts_mut(ptr, size))
}
pub const fn null<T: ?Sized>() -> *const T {
    unsafe {transmute_unchecked_unsized((0usize, 0usize))}
}
pub const fn null_mut<T: ?Sized>() -> *mut T {
    unsafe {transmute_unchecked_unsized((0usize, 0usize))}
}