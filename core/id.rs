use thiserror::Error;

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
#[error("parse uid error")]
pub struct ParseUidError;

impl std::str::FromStr for Id {
	type Err = ParseUidError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self(
			u128::from_str_radix(s, 16).map_err(|_| ParseUidError)?,
		))
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

impl<'a> postgres_types::FromSql<'a> for Id {
	fn from_sql(
		_: &postgres_types::Type,
		raw: &[u8],
	) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
		Ok(std::str::from_utf8(raw)?.parse()?)
	}
	postgres_types::accepts!(BPCHAR);
}

impl postgres_types::ToSql for Id {
	fn to_sql(
		&self,
		_: &postgres_types::Type,
		w: &mut bytes::BytesMut,
	) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
		bytes::BufMut::put_slice(w, self.to_string().as_bytes());
		Ok(postgres_types::IsNull::No)
	}
	postgres_types::accepts!(BPCHAR);
	postgres_types::to_sql_checked!();
}

// impl sqlx::Type<sqlx::Any> for Id {
// 	fn type_info() -> sqlx::any::AnyTypeInfo {
// 		<String as sqlx::Type<sqlx::Any>>::type_info()
// 	}
// }

// impl<'q> sqlx::Encode<'q, sqlx::Any> for Id {
// 	fn encode(self, args: &mut sqlx::any::AnyArgumentBuffer<'q>) -> sqlx::encode::IsNull {
// 		<String as sqlx::Encode<'q, sqlx::Any>>::encode(self.to_string(), args)
// 	}
// 	fn encode_by_ref(&self, args: &mut sqlx::any::AnyArgumentBuffer<'q>) -> sqlx::encode::IsNull {
// 		<String as sqlx::Encode<'q, sqlx::Any>>::encode_by_ref(&self.to_string(), args)
// 	}
// }

// impl<'r> sqlx::Decode<'r, sqlx::Any> for Id {
// 	fn decode(value: sqlx::any::AnyValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
// 		let string = <String as sqlx::Decode<'r, sqlx::Any>>::decode(value)?;
// 		let id = string.parse()?;
// 		Ok(id)
// 	}
// }

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
