use crate::util::progress_counter::ProgressCounter;

#[derive(Clone, Debug)]
pub enum Progress {
	Initializing(ProgressCounter),
	Training(ProgressCounter),
}
