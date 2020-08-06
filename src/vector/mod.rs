pub mod vec2;
pub mod vec3;

use std::ops::{Add, Mul, Sub};

pub trait VecMethods<T> {
    fn sum(&self) -> T;
    fn get_attenuation_factor(&self) -> T;
}

pub trait VecType<T>:
    VecMethods<T>
    + Copy
    + Sub<Self, Output = Self>
    + Add<Self, Output = Self>
    + Mul<T, Output = Self>
    + std::marker::Sized
{
}
impl<
        T,
        X: VecMethods<T>
            + Copy
            + Sub<Self, Output = Self>
            + Add<Self, Output = Self>
            + Mul<T, Output = Self>,
    > VecType<T> for X
{
}

trait VecArgumentType:
    Sub<Self, Output = Self> + Add<Self, Output = Self> + Mul<Self, Output = Self> + Copy
{
}
impl<T: Sub<Self, Output = Self> + Add<Self, Output = Self> + Mul<Self, Output = Self> + Copy>
    VecArgumentType for T
{
}
