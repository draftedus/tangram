use tangram_deps::backtrace::Backtrace;

#[macro_export]
macro_rules! err {
	($msg:expr) => {
		::tangram_util::error::Error::new(::tangram_util::error::DisplayError($msg))
	};
	($fmt:expr, $($arg:tt)*) => {
		::tangram_util::error::Error::new(::tangram_util::error::DisplayError(format!($fmt, $($arg)*)))
	};
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Error {
	error: Box<dyn std::error::Error + Send + Sync + 'static>,
	backtrace: Backtrace,
}

impl<E> From<E> for Error
where
	E: std::error::Error + Send + Sync + 'static,
{
	fn from(value: E) -> Self {
		Error::new(value)
	}
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}\n{:?}", self.error, self.backtrace)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Error {
	pub fn new<E>(error: E) -> Error
	where
		E: std::error::Error + Send + Sync + 'static,
	{
		let backtrace = Backtrace::new();
		Self {
			error: Box::new(error),
			backtrace,
		}
	}

	pub fn error(&self) -> &(dyn std::error::Error + 'static) {
		self.error.as_ref()
	}

	pub fn backtrace(&self) -> &Backtrace {
		&self.backtrace
	}

	pub fn downcast<T>(mut self) -> Result<T, Self>
	where
		T: std::error::Error + Send + Sync + 'static,
	{
		self.error = match self.error.downcast() {
			Ok(error) => return Ok(*error),
			Err(error) => error,
		};
		Err(self)
	}

	pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
	where
		T: std::error::Error + Send + Sync + 'static,
	{
		self.error.downcast_mut()
	}

	pub fn downcast_ref<T>(&self) -> Option<&T>
	where
		T: std::error::Error + Send + Sync + 'static,
	{
		self.error.downcast_ref()
	}
}

#[repr(transparent)]
pub struct DisplayError<T>(pub T);

impl<T> std::fmt::Debug for DisplayError<T>
where
	T: std::fmt::Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		std::fmt::Display::fmt(&self.0, f)
	}
}

impl<T> std::fmt::Display for DisplayError<T>
where
	T: std::fmt::Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		std::fmt::Display::fmt(&self.0, f)
	}
}

impl<T> std::error::Error for DisplayError<T> where T: std::fmt::Display {}
