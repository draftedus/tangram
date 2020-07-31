use crate::util::progress_counter::ProgressCounter;

#[derive(Debug)]
pub enum Progress {
	Loading(ProgressCounter),
	Shuffling,
	Stats(StatsProgress),
	Training(usize, Option<TrainProgress>),
	Testing(ProgressCounter),
}

#[derive(Debug)]
pub enum StatsProgress {
	DatasetStats(ProgressCounter),
	HistogramStats(ProgressCounter),
}

#[derive(Debug)]
pub enum TrainProgress {
	ComputingFeatures(ProgressCounter),
	TrainingModel(ModelTrainProgress),
	ComputingModelComparisonMetrics(ProgressCounter),
}

#[derive(Clone, Debug)]
pub enum ModelTrainProgress {
	Linear(crate::linear::Progress),
	GBT(crate::gbt::Progress),
}
