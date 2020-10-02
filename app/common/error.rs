#[derive(Debug)]
pub enum Error {
	BadRequest,
	Unauthorized,
	NotFound,
	ServiceUnavailable,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Error::BadRequest => "bad request",
			Error::Unauthorized => "unauthorized",
			Error::NotFound => "not found",
			Error::ServiceUnavailable => "service unavailable",
		};
		write!(f, "{}", s)
	}
}

impl std::error::Error for Error {}
