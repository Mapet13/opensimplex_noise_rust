use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Point<T>
where
    T: Add<Output = T>,
    T: Copy,
{
    pub fn sum(&self) -> T {
        self.x + self.y
    }
}

impl<T: Copy> Point<T>
{
    pub fn map<Y>(&self, f: impl Fn(T) -> Y) -> Point<Y> {
        Point {
            x: f(self.x),
            y: f(self.y),
        }
    }
}

impl<T> Sub<Point<T>> for Point<T>
where
    T: Sub<Output = T>,
{
    type Output = Point<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Add<Output = T>> Add<Point<T>> for Point<T>
{
    type Output = Point<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Mul<T> for Point<T>
where
    T: Mul<Output = T>,
    T: Copy,
{
    type Output = Point<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
