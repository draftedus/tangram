use thiserror::Error;

/**
Tangram uses this `Id` type to uniquely identify models, users, and anything that needs a primary key. This type is almost identical to a UUID v4, except there are no bits reserved to specify the version, and the string representation has no dashes. We do not like dashes in our unique identifiers.
*/
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Id(u128);

impl Id {
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for Id {
	fn default() -> Self {
		Self(rand::random())
	}
}

#[derive(Debug, Error)]
#[error("parse id error")]
pub struct ParseIdError;

impl std::str::FromStr for Id {
	type Err = ParseIdError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self(u128::from_str_radix(s, 16).map_err(|_| ParseIdError)?))
	}
}

impl std::fmt::Display for Id {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:032x?}", self.0)
	}
}

impl serde::Serialize for Id {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

struct IdVisitor;

impl<'de> serde::de::Visitor<'de> for IdVisitor {
	type Value = Id;
	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		formatter.write_str("a string")
	}
	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		Ok(value.parse().map_err(|_| E::custom("invalid id"))?)
	}
}

impl<'de> serde::Deserialize<'de> for Id {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		deserializer.deserialize_str(IdVisitor)
	}
}

#[test]
fn test_parse() {
	let s = "00000000000000000000000000000000";
	assert_eq!(s.parse::<Id>().unwrap().to_string(), s);
	let s = "0000000000000000000000000000000z";
	assert!(s.parse::<Id>().is_err());
	let s = "f51a3a61ee9d4731b1b06c816a8ab856";
	assert_eq!(s.parse::<Id>().unwrap().to_string(), s);
	let s = "hello world";
	assert!(s.parse::<Id>().is_err());
}
