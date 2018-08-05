use std::borrow::{Borrow, BorrowMut};
use std::cmp;
use std::fmt;
use std::hash::{Hash, Hasher};
#[cfg(feature = "std")]
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::iter::{FusedIterator, TrustedLen};

use RefMove;

impl<'a, T: ?Sized + 'a> Borrow<T> for RefMove<'a, T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<'a, T: ?Sized + 'a> BorrowMut<T> for RefMove<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T: fmt::Write + ?Sized> fmt::Write for RefMove<'a, T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        <T as fmt::Write>::write_str(self, s)
    }
    fn write_char(&mut self, c: char) -> fmt::Result {
        <T as fmt::Write>::write_char(self, c)
    }
    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        <T as fmt::Write>::write_fmt(self, args)
    }
}

impl<'a, T: ?Sized + 'a> fmt::Pointer for RefMove<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr, f)
    }
}

macro_rules! delegate_format {
    ($Trait:path) => {
        impl<'a, T: $Trait + ?Sized + 'a> $Trait for RefMove<'a, T> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                <T as $Trait>::fmt(self, f)
            }
        }
    };
}
delegate_format!(fmt::Debug);
delegate_format!(fmt::Display);
delegate_format!(fmt::Binary);
delegate_format!(fmt::Octal);
delegate_format!(fmt::LowerHex);
delegate_format!(fmt::UpperHex);
delegate_format!(fmt::LowerExp);
delegate_format!(fmt::UpperExp);

macro_rules! delegate_partial_ord {
    ($a:lifetime, $b:lifetime, $A:ident, $B:ident, $RefA:ty, $RefB:ty) => {
        impl<$a, $b, $A, $B> PartialEq<$RefB> for $RefA
        where
            $A: PartialEq<$B> + ?Sized,
            $B: ?Sized,
        {
            fn eq(&self, other: &$RefB) -> bool {
                <$A as PartialEq<$B>>::eq(self, other)
            }
            fn ne(&self, other: &$RefB) -> bool {
                <$A as PartialEq<$B>>::ne(self, other)
            }
        }

        impl<$a, $b, $A, $B> PartialOrd<$RefB> for $RefA
        where
            $A: PartialOrd<$B> + ?Sized,
            $B: ?Sized,
        {
            fn partial_cmp(&self, other: &$RefB) -> Option<cmp::Ordering> {
                <$A as PartialOrd<$B>>::partial_cmp(self, other)
            }
            fn lt(&self, other: &$RefB) -> bool {
                <$A as PartialOrd<$B>>::lt(self, other)
            }
            fn le(&self, other: &$RefB) -> bool {
                <$A as PartialOrd<$B>>::le(self, other)
            }
            fn gt(&self, other: &$RefB) -> bool {
                <$A as PartialOrd<$B>>::gt(self, other)
            }
            fn ge(&self, other: &$RefB) -> bool {
                <$A as PartialOrd<$B>>::ge(self, other)
            }
        }
    };
}
delegate_partial_ord!('a, 'b, A, B, RefMove<'a, A>, RefMove<'b, B>);
delegate_partial_ord!('a, 'b, A, B, RefMove<'a, A>, &'b mut B);
delegate_partial_ord!('a, 'b, A, B, RefMove<'a, A>, &'b B);
// delegate_partial_ord!('a, 'b, A, B, &'a mut A, RefMove<'b, B>);
// delegate_partial_ord!('a, 'b, A, B, &'a A, RefMove<'b, B>);

impl<'a, A> Eq for RefMove<'a, A> where A: Eq + ?Sized {}
impl<'a, A> Ord for RefMove<'a, A>
where
    A: Ord + ?Sized,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        <A as Ord>::cmp(self, other)
    }
}

impl<'a, T, U> AsRef<U> for RefMove<'a, T>
where
    T: AsRef<U> + ?Sized,
    U: ?Sized,
{
    fn as_ref(&self) -> &U {
        (self as &T).as_ref()
    }
}
impl<'a, T, U> AsMut<U> for RefMove<'a, T>
where
    T: AsMut<U> + ?Sized,
    U: ?Sized,
{
    fn as_mut(&mut self) -> &mut U {
        (self as &mut T).as_mut()
    }
}

impl<'a, A, F> FnOnce<A> for RefMove<'a, F>
where
    F: FnMut<A> + ?Sized, // TODO
{
    type Output = F::Output;
    extern "rust-call" fn call_once(mut self, args: A) -> F::Output {
        <F as FnMut<A>>::call_mut(&mut self, args)
    }
}

impl<'a, A, F> FnMut<A> for RefMove<'a, F>
where
    F: FnMut<A> + ?Sized,
{
    extern "rust-call" fn call_mut(&mut self, args: A) -> F::Output {
        <F as FnMut<A>>::call_mut(self, args)
    }
}

impl<'a, A, F> Fn<A> for RefMove<'a, F>
where
    F: Fn<A> + ?Sized,
{
    extern "rust-call" fn call(&self, args: A) -> F::Output {
        <F as Fn<A>>::call(self, args)
    }
}

impl<'a, T: Hash + ?Sized> Hash for RefMove<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::hash(self, state)
    }
}

impl<'a, I: Iterator + ?Sized> Iterator for RefMove<'a, I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        I::next(self)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        I::size_hint(self)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        I::nth(self, n)
    }
}
impl<'a, I: DoubleEndedIterator + ?Sized> DoubleEndedIterator for RefMove<'a, I> {
    fn next_back(&mut self) -> Option<I::Item> {
        I::next_back(self)
    }
}
impl<'a, I: ExactSizeIterator + ?Sized> ExactSizeIterator for RefMove<'a, I> {
    fn len(&self) -> usize {
        I::len(self)
    }
    fn is_empty(&self) -> bool {
        I::is_empty(self)
    }
}
impl<'a, I: FusedIterator + ?Sized> FusedIterator for RefMove<'a, I> {}
unsafe impl<'a, I: TrustedLen + ?Sized> TrustedLen for RefMove<'a, I> {}

#[cfg(feature = "std")]
impl<'a, W: Write + ?Sized> Write for RefMove<'a, W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        W::write(self, buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        W::flush(self)
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        W::write_all(self, buf)
    }
    #[inline]
    fn write_fmt(&mut self, f: fmt::Arguments) -> io::Result<()> {
        W::write_fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl<'a, R: Read + ?Sized> Read for RefMove<'a, R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        R::read(self, buf)
    }
    #[inline]
    unsafe fn initializer(&self) -> io::Initializer {
        R::initializer(self)
    }
    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        R::read_to_end(self, buf)
    }
    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        R::read_to_string(self, buf)
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        R::read_exact(self, buf)
    }
}
#[cfg(feature = "std")]
impl<'a, S: Seek + ?Sized> Seek for RefMove<'a, S> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        S::seek(self, pos)
    }
}
#[cfg(feature = "std")]
impl<'a, B: BufRead + ?Sized> BufRead for RefMove<'a, B> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        B::fill_buf(self)
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        B::consume(self, amt)
    }
    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        B::read_until(self, byte, buf)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        B::read_line(self, buf)
    }
}
