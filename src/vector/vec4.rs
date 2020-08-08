use std::ops::{Add, Mul, Sub};

use super::VecMethods;

#[derive(Copy, Clone, Debug)]
pub struct Vec4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T> Vec4<T> {
    pub const fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }
}

impl<T> VecMethods<T> for Vec4<T>
where
    T: Add<Output = T>,
    T: Mul<Output = T>,
    T: Sub<Output = T>,
    T: Copy,
{
    fn sum(&self) -> T {
        self.x + self.y + self.z + self.w
    }

    fn get_attenuation_factor(&self) -> T {
        (self.x * self.x) + (self.y * self.y) + (self.z * self.z) + (self.w * self.w)
    }
}

impl<T: Copy> Vec4<T> {
    pub fn map<Y>(&self, f: impl Fn(T) -> Y) -> Vec4<Y> {
        Vec4 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
            w: f(self.w),
        }
    }
}

impl<T> Sub<Vec4<T>> for Vec4<T>
where
    T: Sub<Output = T>,
{
    type Output = Vec4<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl<T: Add<Output = T>> Add<Vec4<T>> for Vec4<T> {
    type Output = Vec4<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl<T> Mul<T> for Vec4<T>
where
    T: Mul<Output = T>,
    T: Copy,
{
    type Output = Vec4<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Vec4 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}
