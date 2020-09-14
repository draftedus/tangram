
#[derive(Clone, Debug)]
pub enum Progress {
	Initializing(tangram_progress::ProgressCounter),
	Training(tangram_progress::ProgressCounter),
}
