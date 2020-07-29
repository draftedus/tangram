use num_traits::Float;
use std::{
	cmp::{Ord, Ordering},
	fmt::Debug,
	hash::{Hash, Hasher},
	ops::{Add, Mul, Sub},
};
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub struct Finite<T>(T)
where
	T: Float;

#[derive(Debug, Error)]
#[error("not finite")]
pub struct NotFiniteError;

impl<T> Finite<T>
where
	T: Float,
{
	pub fn new(value: T) -> Result<Self, NotFiniteError> {
		if value.is_finite() {
			Ok(Self(value))
		} else {
			Err(NotFiniteError)
		}
	}

	pub fn get(self) -> T {
		self.0
	}
}

impl<T> std::ops::Deref for Finite<T>
where
	T: Float,
{
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> std::ops::DerefMut for Finite<T>
where
	T: Float,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T> std::fmt::Display for Finite<T>
where
	T: Float + std::fmt::Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl<T> PartialEq for Finite<T>
where
	T: Float,
{
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl<T> Eq for Finite<T> where T: Float {}

impl<T> PartialOrd for Finite<T>
where
	T: Float,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<T> Ord for Finite<T>
where
	T: Float,
{
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.partial_cmp(&other.0).unwrap()
	}
}

impl Hash for Finite<f32> {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.to_bits().hash(state);
	}
}

impl Hash for Finite<f64> {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.to_bits().hash(state);
	}
}

impl<T> Add for Finite<T>
where
	T: Float,
{
	type Output = Self;
	fn add(self, other: Self) -> Self::Output {
		Self::new(self.0.add(other.0)).unwrap()
	}
}

impl<T> Add<T> for Finite<T>
where
	T: Float,
{
	type Output = Self;
	fn add(self, other: T) -> Self::Output {
		Self::new(self.0.add(other)).unwrap()
	}
}

impl<T> Sub for Finite<T>
where
	T: Float,
{
	type Output = Self;
	fn sub(self, other: Self) -> Self::Output {
		Self::new(self.0.sub(other.0)).unwrap()
	}
}

impl<T> Sub<T> for Finite<T>
where
	T: Float,
{
	type Output = Self;
	fn sub(self, other: T) -> Self::Output {
		Self::new(self.0.sub(other)).unwrap()
	}
}

impl<T> Mul for Finite<T>
where
	T: Float,
{
	type Output = Self;
	fn mul(self, other: Self) -> Self::Output {
		Self::new(self.0.mul(other.0)).unwrap()
	}
}

impl<T> Mul<T> for Finite<T>
where
	T: Float,
{
	type Output = Self;
	fn mul(self, other: T) -> Self::Output {
		Self::new(self.0.mul(other)).unwrap()
	}
}

pub trait ToFinite<T>
where
	T: Float,
{
	/// if the value is finite, return Some(Finite(self)), otherwise return None.
	fn to_finite(self) -> Result<Finite<T>, NotFiniteError>;
}

impl<T> ToFinite<T> for T
where
	T: Float,
{
	fn to_finite(self) -> Result<Finite<T>, NotFiniteError> {
		Finite::new(self)
	}
}
