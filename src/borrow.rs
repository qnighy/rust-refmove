use std::borrow::BorrowMut;
use std::ops::Deref;

use RefMove;
use anchor::{Anchor, StackAnchor, IdentityAnchor};
#[cfg(feature = "std")]
use anchor::BoxAnchor;

pub trait BorrowInterior<Borrowed: ?Sized>: BorrowMut<Borrowed> + Sized {
    type Anchor: Anchor<Self, Borrowed>;
}

impl<T> BorrowInterior<T> for T {
    type Anchor = StackAnchor<T>;
}

impl<'a, T: ?Sized> BorrowInterior<T> for RefMove<'a, T> {
    type Anchor = IdentityAnchor<'a, T>;
}

// TODO: make T: ?Sized once rust-lang/rust#53033 lands
#[cfg(feature = "std")]
impl<T> BorrowInterior<T> for Box<T> {
    type Anchor = BoxAnchor<T>;
}

pub trait AnchorExt: Sized {
    fn anchor(self) -> StackAnchor<Self> {
        StackAnchor::anchor_from(self)
    }

    fn anchor_box(self) -> Self::Anchor
    where
        Self: Deref,
        Self: BorrowInterior<<Self as Deref>::Target>,
    {
        Self::Anchor::anchor_from(self)
    }
}

impl<T> AnchorExt for T {}

pub trait BorrowMove<Borrowed: ?Sized>: BorrowMut<Borrowed> {
    fn borrow_move<'a>(self: RefMove<'a, Self>) -> RefMove<'a, Borrowed>;
}

impl<T: ?Sized> BorrowMove<T> for T {
    fn borrow_move<'a>(self: RefMove<'a, Self>) -> RefMove<'a, T> {
        self
    }
}

impl<'a, T: ?Sized> BorrowMove<T> for RefMove<'a, T> {
    fn borrow_move<'b>(self: RefMove<'b, Self>) -> RefMove<'b, T> {
        self.into_inner()
    }
}

macro_rules! define_array_borrow {
    ($($n:expr),*) => {
        $(
            impl<T> BorrowMove<[T]> for [T; $n] {
                fn borrow_move<'a>(self: RefMove<'a, Self>) -> RefMove<'a, [T]> {
                    self
                }
            }
        )*
    };
}

define_array_borrow!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
define_array_borrow!(16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32);
