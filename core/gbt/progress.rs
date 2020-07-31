use crate::util::progress_counter::ProgressCounter;

#[derive(Clone, Debug)]
pub enum Progress {
	Initializing(InitializingProgress),
	Training(TrainingProgress),
}

#[derive(Clone, Debug)]
pub struct InitializingProgress {
	pub feature: ProgressCounter,
}

#[derive(Clone, Debug)]
pub struct TrainingProgress {
	pub round: ProgressCounter,
}
