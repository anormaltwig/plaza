use std::ops::{Add, Sub};

#[derive(Clone)]
pub struct Vector3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Vector3 {
	pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
		Vector3 { x, y, z }
	}

	pub fn set(&mut self, x: f32, y: f32, z: f32) {
		self.x = x;
		self.y = y;
		self.z = z;
	}

	pub fn length_sqr(&self) -> f32 {
		self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
	}

	pub fn length(&self) -> f32 {
		self.length_sqr().sqrt()
	}

	pub fn distance_sqr(&self, other: &Self) -> f32 {
		(other - self).length_sqr()
	}

	pub fn distance(&self, other: &Self) -> f32 {
		(other - self).length()
	}
}

impl Add<&Vector3> for &Vector3 {
	type Output = Vector3;

	fn add(self, rhs: &Vector3) -> Vector3 {
		Vector3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
	}
}

impl Sub<&Vector3> for &Vector3 {
	type Output = Vector3;

	fn sub(self, rhs: &Vector3) -> Vector3 {
		Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
	}
}

#[derive(Clone)]
pub struct Mat3 {
	pub data: [f32; 9],
}

impl Mat3 {
	pub fn new() -> Mat3 {
		Mat3 { data: [0.0; 9] }
	}
}
