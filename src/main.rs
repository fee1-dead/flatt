#![feature(
    offset_of,
    generic_const_exprs,
    macro_metavar_expr,
    maybe_uninit_uninit_array
)]

use std::marker::PhantomData;
use std::mem::{align_of, offset_of, size_of, ManuallyDrop, MaybeUninit};
use std::ops::Deref;
use std::ptr::addr_of;

use flatt::Inaccessible;

#[derive(flatt::Embeddable)]
#[flatt(HasFoo, FooContainer)]
pub struct Foo<A> {
    pub x: A,
    pub y: i32,
}

#[repr(transparent)]
pub struct FakeBorrow<'a> {
    _p: PhantomData<&'a ()>,
}

impl<'a> FakeBorrow<'a> {
    pub fn new<T>(_: &'a T) -> Self {
        Self { _p: PhantomData }
    }

    pub fn combine<U>(self, _: &'a U) -> FakeBorrow<'a> {
        FakeBorrow { _p: PhantomData }
    }
}

impl<'a> Drop for FakeBorrow<'a> {
    fn drop(&mut self) {}
}

/// Force a reference to follow the lifetime of a fake borrow
pub struct Guard<'a, T> {
    x: &'a T,
    fb: FakeBorrow<'a>,
}

impl<'a, T> Guard<'a, T> {
    pub fn new(x: &'a T, fb: FakeBorrow<'a>) -> Self {
        Self { x, fb }
    }
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.x
    }
}

pub struct Hihi {
    x: f32,
    y: i32,
    z: i32,
}

trait TypeData {
    const VALUE: usize;
}

trait TypeOp {
    type Result;
}

pub struct TypeVal<T, const VALUE: usize>(T);

impl<T, const VALUE: usize> TypeData for TypeVal<T, VALUE> {
    const VALUE: usize = VALUE;
}

impl<T, const VALUE: usize> TypeOp for TypeVal<T, VALUE> {
    type Result = T;
}

struct Choose<A: TypeData, B: TypeData, const RIGHT: bool>(A, B);

impl<A: TypeData, B: TypeData> TypeOp for Choose<A, B, false> {
    type Result = A;
}

impl<A: TypeData, B: TypeData> TypeOp for Choose<A, B, true> {
    type Result = B;
}

struct Min<A: TypeData, B: TypeData>(A, B);

impl<A: TypeData, B: TypeData> TypeOp for Min<A, B>
where
    Choose<A, B, { A::VALUE > B::VALUE }>: TypeOp,
{
    type Result = <Choose<A, B, { A::VALUE > B::VALUE }> as TypeOp>::Result;
}

/// Find the type with the minimum value that is greater than a lower bound.
struct MinGreaterThan<A: TypeData, B: TypeData, const LOWER_BOUND: usize>(A, B);

/// Choose the other type when either type does not meet the bound requirement
///
/// If A is greater than B, choose B.
const fn choose<A: TypeData, B: TypeData>(lower_bound: usize) -> bool {
    if A::VALUE <= lower_bound {
        true
    } else if B::VALUE <= lower_bound {
        false
    } else {
        A::VALUE > B::VALUE
    }
}

impl<A: TypeData, B: TypeData, const LOWER_BOUND: usize> TypeOp
    for MinGreaterThan<A, B, LOWER_BOUND>
where
    Choose<A, B, { choose::<A, B>(LOWER_BOUND) }>: TypeOp,
{
    type Result = <Choose<A, B, { choose::<A, B>(LOWER_BOUND) }> as TypeOp>::Result;
}

macro_rules! sort_types {
    ([$ty1:ty, $($ty:ty),*$(,)?]$([])?) => {
        $(${ignore(ty)} <Min<)* $ty1 $(, $ty> as TypeOp>::Result)*
    };
    ([$ty1:ty, $($ty:ty),*$(,)?][][$e:expr]) => {
        $(${ignore(ty)} <MinGreaterThan<)* $ty1 $(, $ty, $e> as TypeOp>::Result)*
    };
    ([$ty1:ty, $($ty:ty),*$(,)?][$tt:tt $($tts:tt)*]) => {
        sort_types!([$ty1, $($ty),*][][{<sort_types!([$ty1, $($ty),*][$($tts)*]) as TypeData>::VALUE}])
    };
}
pub fn a() {
    let a: <sort_types!([TypeVal<u32, 2>, TypeVal<u16, 1>,]) as TypeOp>::Result = 1u16;
    let a: <sort_types!([TypeVal<u32, 2>, TypeVal<u16, 1>,][+]) as TypeOp>::Result = 1u32;
}

pub fn takes(mut x: Hihi) {
    let x_ref = &*Guard::new(
        unsafe { addr_of!(x.x).as_ref().unwrap() },
        FakeBorrow::new(&x.x),
    );
}

/// Map an expression of type `T: ContainsTe<A>`
macro_rules! take_te_ref {
    ($val: expr) => {};
}

#[repr(C)]
pub struct XAccessor__<const P__: usize, const A__: usize, A> {
    __p: [MaybeUninit<u8>; P__],
    x: A,
    __a: [MaybeUninit<u8>; A__],
}

macro_rules! Bar {
    (pub struct $name:ident { $($tt:tt)* }) => {
        pub struct $name {
            pub __x: i32,
            pub __y: i32,
            $($tt)*
        }

        #[repr(C)]
        pub struct X {
            uh: [MaybeUninit<u8>; offset_of!($name, __x)],
            pub x: i32,
            uhh: [MaybeUninit<u8>; size_of::<$name>() - size_of::<i32>() - offset_of!($name, __x)],
        }

        #[repr(C)]
        pub struct Y {
            uh: [MaybeUninit<u8>; offset_of!($name, __y)],
            pub y: i32,
            uhh: [MaybeUninit<u8>; size_of::<$name>() - size_of::<i32>() - offset_of!($name, __y)],
        }

        #[repr(transparent)]
        pub struct BarWrapper {
            pub bar: X,
        }

        impl std::ops::Deref for $name {
            type Target = BarWrapper;
            fn deref(&self) -> &BarWrapper {
                unsafe {
                    <*const $name>::from(self).cast::<BarWrapper>().as_ref().unwrap()
                }
            }
        }

        impl std::ops::Deref for X {
            type Target = Y;

            fn deref(&self) -> &Y {
                unsafe {
                    <*const X>::from(self).cast::<Y>().as_ref().unwrap()
                }
            }
        }

        impl X {
            pub fn some_method(&self) -> i32 {
                self.x * self.y
            }
        }
    };
}

fn main() {}
