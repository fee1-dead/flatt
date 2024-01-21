use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub use flatt_derive::Embeddable;

mod sealed {
    pub trait Sealed {}

    impl<T, const OFFSET: usize> Sealed for super::FieldReader<T, OFFSET> {}
}

#[repr(transparent)]
pub struct Inaccessible<T> {
    _x: ManuallyDrop<T>,
}

impl<T> Inaccessible<T> {
    pub const fn new(x: T) -> Self {
        Self {
            _x: ManuallyDrop::new(x),
        }
    }
}

#[repr(transparent)]
pub struct FieldReader<T, const OFFSET: usize> {
    ph: PhantomData<T>,
}

impl<T, const OFFSET: usize> FieldReader<T, OFFSET> {
    /// For this to be safe any reference to `FieldReader` must have
    /// the same address as a type containing the field at `OFFSET` with
    /// type `T`. This means that `FieldReader` cannot be moved out of any
    /// type that contains it. To achieve this, any type containing this type
    /// must implement `Drop` to prevent this field from being moved out of,
    /// and must be `repr(C)`.
    pub const unsafe fn new() -> Self {
        Self { ph: PhantomData }
    }
}

pub trait IsFieldReader: sealed::Sealed {
    type Type;
    const OFFSET: usize;
}

impl<T, const OFFSET: usize> IsFieldReader for FieldReader<T, OFFSET> {
    type Type = T;
    const OFFSET: usize = OFFSET;
}

impl<T, const OFFSET: usize> Deref for FieldReader<T, OFFSET> {
    type Target = T;
    fn deref(&self) -> &T {
        let ptr = <*const Self>::from(self);
        let ptr = unsafe { ptr.cast::<u8>().add(OFFSET).cast::<T>() };
        unsafe { &*ptr }
    }
}

impl<T, const OFFSET: usize> DerefMut for FieldReader<T, OFFSET> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = <*mut Self>::from(self);
        let ptr = unsafe { ptr.cast::<u8>().add(OFFSET).cast::<T>() };
        unsafe { &mut *ptr }
    }
}

#[repr(transparent)]
pub struct Zstizer<T: IsFieldReader> {
    pub ph: PhantomData<T>,
}

impl<T: IsFieldReader> Zstizer<T> {
    pub const unsafe fn new() -> Self {
        Self { ph: PhantomData }
    }
}

impl<T: IsFieldReader> Deref for Zstizer<T> {
    type Target = T::Type;
    fn deref(&self) -> &Self::Target {
        let ptr = <*const Self>::from(self);
        let ptr = unsafe { ptr.cast::<u8>().add(T::OFFSET).cast::<T::Type>() };
        unsafe { &*ptr }
    }
}

impl<T: IsFieldReader> DerefMut for Zstizer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = <*mut Self>::from(self);
        let ptr = unsafe { ptr.cast::<u8>().add(T::OFFSET).cast::<T::Type>() };
        unsafe { &mut *ptr }
    }
}
