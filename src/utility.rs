use std::ops;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Number<T>
where
    T: Sized,
{
    pub val: T,
}

impl<T> ops::Add for Number<T>
where
    T: ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Number {
            val: self.val + rhs.val,
        }
    }
}

impl<T> ops::Sub for Number<T>
where
    T: ops::Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Number {
            val: self.val - rhs.val,
        }
    }
}

impl<T> ops::Mul for Number<T>
where
    T: ops::Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Number {
            val: self.val * rhs.val,
        }
    }
}

impl<T> ops::Div for Number<T>
where
    T: ops::Div<Output = T>,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Number {
            val: self.val / rhs.val,
        }
    }
}
