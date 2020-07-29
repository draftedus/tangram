use derive_more::{Display, Error};

#[derive(Display, Debug, Error)]
pub enum Error {
	BadRequest,
	Unauthorized,
	NotFound,
	ServiceUnavailable,
}
