use std::mem::ManuallyDrop;
use std::fmt;

use RefMove;

pub trait Anchor<T, U: ?Sized> {
    fn anchor_from(content: T) -> Self;
    fn borrow_move<'a>(&'a mut self) -> RefMove<'a, U>;
}

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
        self.content.take().expect("double borrow_move from IdentityAnchor")
    }
}
