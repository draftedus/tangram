use crate::util::progress_counter::ProgressCounter;

#[derive(Clone, Debug)]
pub struct Progress {
	pub epoch: ProgressCounter,
	pub example: ProgressCounter,
}
