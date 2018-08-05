//! Anchors
//!
//! Anchors ensure validity of memory regions at caller side.

use std::fmt;
use std::mem::ManuallyDrop;

use RefMove;

/// Anchors
///
/// Anchors ensure validity of memory regions at caller side.
pub trait Anchor<T, U: ?Sized> {
    /// Wraps the ownership by this anchor.
    fn anchor_from(content: T) -> Self;
    /// Turns a mutable reference to this anchor into a by-move reference
    /// to its content.
    ///
    /// ## Panics
    ///
    /// This method panics when called more than once.
    fn borrow_move<'a>(&'a mut self) -> RefMove<'a, U>;
}

/// Anchor to obtain by-move reference to the stack.
///
/// The structure is similar to `Option<T>` but avoids some unwanted
/// optimizations for this purpose.
pub struct StackAnchor<T> {
    is_some: bool,
    content: ManuallyDrop<T>,
}

impl<T> Anchor<T, T> for StackAnchor<T> {
    fn anchor_from(content: T) -> Self {
        Self {
            is_some: true,
            content: ManuallyDrop::new(content),
        }
    }

    fn borrow_move<'a>(&'a mut self) -> RefMove<'a, T> {
        assert!(self.is_some, "double borrow_move from StackAnchor");
        self.is_some = false;
        unsafe { RefMove::from_mut(&mut self.content) }
    }
}

unsafe impl<#[may_dangle] T> Drop for StackAnchor<T> {
    fn drop(&mut self) {
        unsafe {
            if self.is_some {
                ManuallyDrop::drop(&mut self.content);
            }
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for StackAnchor<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_some {
            f.debug_struct("StackAnchor")
                .field("is_some", &self.is_some)
                .field("content", &self.content as &T)
                .finish()
        } else {
            f.debug_struct("StackAnchor")
                .field("is_some", &self.is_some)
                .finish()
        }
    }
}

// TODO: make T: ?Sized once rust-lang/rust#53033 lands
/// Anchor to obtain by-move reference to the heap.
///
/// The structure is similar to `Box<Option<T>>` but `is_some` flag is
/// out of `Box` so that we can reuse `Box<T>` pointer.
///
/// ## Sizedness
///
/// This type currently imposes the `T: Sized` bound.
/// `T: ?Sized` is just blocked by [#53033][#53033] in the compiler.
///
/// [#53033]: https://github.com/rust-lang/rust/pull/53033
#[cfg(feature = "std")]
pub struct BoxAnchor<T> {
    is_some: bool,
    content: Box<ManuallyDrop<T>>,
}

#[cfg(feature = "std")]
impl<T> Anchor<Box<T>, T> for BoxAnchor<T> {
    fn anchor_from(content: Box<T>) -> Self {
        Self {
            is_some: true,
            content: unsafe { Box::from_raw(Box::into_raw(content) as *mut ManuallyDrop<T>) },
        }
    }

    fn borrow_move<'a>(&'a mut self) -> RefMove<'a, T> {
        assert!(self.is_some, "double borrow_move from BoxAnchor");
        self.is_some = false;
        unsafe { RefMove::from_mut(&mut self.content) }
    }
}

#[cfg(feature = "std")]
unsafe impl<#[may_dangle] T> Drop for BoxAnchor<T> {
    fn drop(&mut self) {
        unsafe {
            if self.is_some {
                ManuallyDrop::drop(&mut self.content);
            }
        }
    }
}

#[cfg(feature = "std")]
impl<T: fmt::Debug> fmt::Debug for BoxAnchor<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_some {
            f.debug_struct("BoxAnchor")
                .field("is_some", &self.is_some)
                .field("content", &self.content as &T)
                .finish()
        } else {
            f.debug_struct("BoxAnchor")
                .field("is_some", &self.is_some)
                .finish()
        }
    }
}

/// Trivial anchor that just returns the given `RefMove`.
#[derive(Debug)]
pub struct IdentityAnchor<'a, T: ?Sized + 'a> {
    content: Option<RefMove<'a, T>>,
}

impl<'a, T: ?Sized + 'a> Anchor<RefMove<'a, T>, T> for IdentityAnchor<'a, T> {
    fn anchor_from(content: RefMove<'a, T>) -> Self {
        Self {
            content: Some(content),
        }
    }

    fn borrow_move<'b>(&'b mut self) -> RefMove<'b, T> {
        self.content
            .take()
            .expect("double borrow_move from IdentityAnchor")
    }
}
