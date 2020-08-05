use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Vec2<T>
where
    T: Add<Output = T>,
    T: Copy,
{
    pub fn sum(&self) -> T {
        self.x + self.y
    }
}

impl<T: Copy> Vec2<T>
{
    pub fn map<Y>(&self, f: impl Fn(T) -> Y) -> Vec2<Y> {
        Vec2 {
            x: f(self.x),
            y: f(self.y),
        }
    }
}

impl<T> Sub<Vec2<T>> for Vec2<T>
where
    T: Sub<Output = T>,
{
    type Output = Vec2<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Add<Output = T>> Add<Vec2<T>> for Vec2<T>
{
    type Output = Vec2<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Mul<T> for Vec2<T>
where
    T: Mul<Output = T>,
    T: Copy,
{
    type Output = Vec2<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}