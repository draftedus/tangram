#[derive(serde::Serialize)]
pub struct Props {
	pub code: bool,
	pub email: Option<String>,
	pub error: Option<String>,
}
