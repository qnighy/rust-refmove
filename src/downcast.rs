use std::any::Any;

use RefMove;

impl<'a> RefMove<'a, dyn Any> {
    /// Attempt to downcast the box to a concrete type.
    pub fn downcast<T: Any>(self) -> Result<RefMove<'a, T>, Self> {
        if self.is::<T>() {
            unsafe {
                let ptr: *mut Any = RefMove::into_ptr(self);
                Ok(RefMove::from_ptr(ptr as *mut T))
            }
        } else {
            Err(self)
        }
    }
}

impl<'a> RefMove<'a, dyn Any + Send> {
    /// Attempt to downcast the box to a concrete type.
    pub fn downcast<T: Any>(self) -> Result<RefMove<'a, T>, Self> {
        if self.is::<T>() {
            unsafe {
                let ptr: *mut Any = RefMove::into_ptr(self);
                Ok(RefMove::from_ptr(ptr as *mut T))
            }
        } else {
            Err(self)
        }
    }
}
