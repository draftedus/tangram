use crate::util::progress_counter::ProgressCounter;

#[derive(Debug)]
pub enum Progress {
	Loading(ProgressCounter),
	Shuffling,
	Stats(StatsProgress),
	Training(GridTrainProgress),
	Testing,
}

#[derive(Debug)]
pub enum StatsProgress {
	DatasetStats(ProgressCounter),
	HistogramStats(ProgressCounter),
}

#[derive(Debug)]
pub struct GridTrainProgress {
	pub current: u64,
	pub total: u64,
	pub grid_item_progress: TrainProgress,
}

#[derive(Debug)]
pub enum TrainProgress {
	ComputingFeatures(ProgressCounter),
	TrainingModel(ModelTrainProgress),
	ComputingModelComparisonMetrics(ModelTestProgress),
}

#[derive(Clone, Debug)]
pub enum ModelTrainProgress {
	Linear(crate::linear::Progress),
	GBT(crate::gbt::Progress),
}

#[derive(Clone, Debug)]
pub enum ModelTestProgress {
	ComputingFeatures(ProgressCounter),
	Testing,
}
