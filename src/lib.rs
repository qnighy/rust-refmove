// We need it to achieve smooth borrowing.
#![feature(nll)]
// To implement Unpin
#![feature(pin)]
// To use #[may_dangle]
#![feature(dropck_eyepatch)]
// To implement CoerceUnsized
#![feature(unsize, coerce_unsized)]
// To use self: RefMove<Self>
#![feature(arbitrary_self_types)]
// To implement FnOnce/FnMut/Fn
#![feature(unboxed_closures, fn_traits)]
// To implement ExactSizeIterator::is_empty
#![feature(exact_size_is_empty)]
// To implement TrustedLen
#![feature(trusted_len)]
// To implement Read::initializer
#![feature(read_initializer)]

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(not(feature = "std"))]
use core as std;

use std::marker::{PhantomData, Unpin, Unsize};
use std::mem::{self, ManuallyDrop};
use std::ops::{CoerceUnsized, Deref, DerefMut};
#[cfg(feature = "std")]
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::ptr::{self, drop_in_place, NonNull};

pub use anchor::Anchor;
pub use borrow::{BorrowInterior, BorrowMove, AnchorExt};

pub mod anchor;
mod borrow;
mod impls;

pub struct RefMove<'a, T: ?Sized + 'a> {
    ptr: NonNull<T>,
    _marker: PhantomData<(&'a (), T)>,
}

// TODO: make T: ?Sized once rust-lang/rust#53033 lands
impl<'a, T: 'a> RefMove<'a, T> {
    pub unsafe fn from_mut(reference: &'a mut ManuallyDrop<T>) -> Self {
        Self {
            ptr: reference.deref_mut().into(),
            _marker: PhantomData,
        }
    }

    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            _marker: PhantomData,
        }
    }
}

impl<'a, T: 'a> RefMove<'a, T> {
    pub fn into_inner(self) -> T {
        let ret = unsafe { ptr::read(self.ptr.as_ptr() as *const T) };
        mem::forget(self);
        ret
    }
}

unsafe impl<'a, T: Send + ?Sized + 'a> Send for RefMove<'a, T> {}
unsafe impl<'a, T: Sync + ?Sized + 'a> Sync for RefMove<'a, T> {}
#[cfg(feature = "std")]
impl<'a, T: UnwindSafe + ?Sized + 'a> UnwindSafe for RefMove<'a, T> {}
#[cfg(feature = "std")]
impl<'a, T: RefUnwindSafe + ?Sized + 'a> RefUnwindSafe for RefMove<'a, T> {}
impl<'a, T: ?Sized + 'a> Unpin for RefMove<'a, T> {}

impl<'a, T: ?Sized + 'a> Deref for RefMove<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a, T: ?Sized + 'a> DerefMut for RefMove<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

unsafe impl<'a, #[may_dangle] T: ?Sized + 'a> Drop for RefMove<'a, T> {
    fn drop(&mut self) {
        unsafe {
            drop_in_place(self.ptr.as_ptr());
        }
    }
}

impl<'a, 'b, T, U> CoerceUnsized<RefMove<'a, U>> for RefMove<'b, T>
where
    'b: 'a,
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: wait for rust-lang/rust#50173

    // use Anchor;
    // use BorrowInteriorExt;

    // trait Foo {
    //     fn foo(self: RefMove<Self>);
    // }

    // impl Foo for String {
    //     fn foo(self: RefMove<Self>) {
    //         println!("{}", self);
    //     }
    // }

    // #[test]
    // fn test_foo() {
    //     let x: Box<Foo> = Box::new("hoge".to_string());
    //     x.anchor_box().borrow_move().foo();
    // }

    #[cfg(feature = "std")]
    #[test]
    fn test_borrow_move() {
        fn f(x: RefMove<String>, e: &str) {
            println!("{}", x);
            assert_eq!(x, e);
        }
        f("hoge".to_string().anchor().borrow_move(), "hoge");
        f(Box::new("fuga".to_string()).anchor_box().borrow_move(), "fuga");
    }

    #[test]
    fn test_borrow_move_nostd() {
        fn f(x: RefMove<&mut i32>) {
            assert_eq!(**x, 42);
        }
        let mut x = 42;
        f((&mut x).anchor().borrow_move());
    }
}
