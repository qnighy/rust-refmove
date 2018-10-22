//! This crate contains an experimental implementation of library-level
//! by-move references.
//!
//! When [#50173][#50173] land in the compiler,
//! it will enable you to use `self: RefMove<Self>` to pass your trait
//! object by value, even without allocation.
//!
//! [#50173]: https://github.com/rust-lang/rust/pull/50173
//!
//! See [#48055][#48055] for another approach to allow by-value trait objects.
//!
//! [#48055]: https://github.com/rust-lang/rust/issues/48055
//!
//! ## Usage
//!
//! ### Borrowing
//!
//! ```rust
//! #![feature(nll)]
//! extern crate refmove;
//! use refmove::{Anchor, AnchorExt, RefMove};
//! # #[cfg(feature = "std")]
//! # fn main() {
//! // Borrowing from stack
//! let _: RefMove<i32> = 42.anchor().borrow_move();
//! // Borrowing from box
//! let _: RefMove<i32> = Box::new(42).anchor_box().borrow_move();
//! # }
//! # #[cfg(not(feature = "std"))]
//! # fn main() {}
//! ```
//!
//! ### Extracting
//!
//! ```rust
//! #![feature(nll)]
//! extern crate refmove;
//! use refmove::{Anchor, AnchorExt, RefMove};
//! # fn main() {
//! #     f("42".to_string().anchor().borrow_move());
//! # }
//! fn f(x: RefMove<String>) {
//!     // Borrowing by dereference
//!     println!("{}", &x as &String);
//!     // Move out ownership
//!     let _: String = x.into_inner();
//! }
//! ```

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
#![cfg_attr(feature = "std", feature(read_initializer))]
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
pub use borrow::{AnchorExt, BorrowInterior, BorrowMove};

pub mod anchor;
mod borrow;
mod downcast;
mod impls;

/// Owned reference.
///
/// `RefMove<'a, T>` inherits both `&'a mut T` and `Box<T>`. For example,
///
/// - As in `RefMove<'a, T>`, `RefMove<'a, T>` is covariant w.r.t. `'a`.
/// - As in `Box<T>`, `RefMove<'a, T>` is covariant w.r.t. `T`.
/// - As in `Box<T>`, `RefMove<'a, T>` has a destructor.
///   However, unlike `Box<T>`, it only destructs its contents and does not
///   free the pointer itself.
/// - Therefore, `RefMove<'a, T>` can refer to the stack like `&'a mut T`.
/// - Like `Box<T>`, you can extract `T` from it.
pub struct RefMove<'a, T: ?Sized + 'a> {
    ptr: NonNull<T>,
    _marker: PhantomData<(&'a (), T)>,
}

impl<'a, T: ?Sized + 'a> RefMove<'a, T> {
    /// Creates `RefMove` from its `ManuallyDrop` reference.
    ///
    /// It works much like [`ManuallyDrop::drop`][ManuallyDrop::drop]
    /// but the actual drop of the content delays until `RefMove` is dropped.
    ///
    /// [ManuallyDrop::drop]: https://doc.rust-lang.org/nightly/core/mem/struct.ManuallyDrop.html#method.drop
    ///
    /// ## Safety
    ///
    /// This function eventually (by expiration of `'a` lifetime) runs
    /// the destructor of the contained value and thus the wrapped value
    /// represents uninitialized data after `'a`.
    /// It is up to the user of this method to ensure the uninitialized data
    /// is actually used.
    pub unsafe fn from_mut(reference: &'a mut ManuallyDrop<T>) -> Self {
        Self {
            ptr: reference.deref_mut().into(),
            _marker: PhantomData,
        }
    }

    /// Creates `RefMove` from a pointer.
    ///
    /// ## Safety
    ///
    /// `ptr` must point to a memory region that is valid until
    /// expiration of `'a` lifetime.
    /// The memory region must not be shared by another.
    /// The memory region must initially contain the valid content.
    /// After expiration of `'a` lifetime, the region is uninitialized.
    pub unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr),
            _marker: PhantomData,
        }
    }

    /// Turns it into a raw pointer, without dropping its content.
    pub fn into_ptr(this: Self) -> *mut T {
        let ptr = this.ptr;
        mem::forget(this);
        ptr.as_ptr()
    }
}

impl<'a, T: 'a> RefMove<'a, T> {
    /// Turns `RefMove` into its content.
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
{
}

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
        f(
            Box::new("fuga".to_string()).anchor_box().borrow_move(),
            "fuga",
        );
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
