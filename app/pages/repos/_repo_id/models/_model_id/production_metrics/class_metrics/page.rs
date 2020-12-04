use tangram_app_common::date_window::{DateWindow, DateWindowInterval};
use tangram_app_layouts::{document::PageInfo, model_layout::ModelLayoutInfo};

pub struct Props {
	pub id: String,
	pub class_metrics: Vec<ClassMetricsEntry>,
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub classes: Vec<String>,
	pub overall: OverallClassMetrics,
	pub model_layout_info: ModelLayoutInfo,
	pub class: String,
}

pub struct ClassMetricsEntry {
	pub class_name: String,
	pub intervals: Vec<IntervalEntry>,
}

pub struct IntervalEntry {
	pub label: String,
	pub f1_score: TrainingProductionMetrics,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
}

pub struct OverallClassMetrics {
	pub class_metrics: Vec<OverallClassMetricsEntry>,
	pub label: String,
}

pub struct OverallClassMetricsEntry {
	pub class_name: String,
	pub comparison: Comparison,
	pub confusion_matrix: ConfusionMatrix,
	pub f1_score: TrainingProductionMetrics,
	pub precision: TrainingProductionMetrics,
	pub recall: TrainingProductionMetrics,
}

pub struct Comparison {
	pub false_negative_fraction: TrainingProductionMetrics,
	pub false_positive_fraction: TrainingProductionMetrics,
	pub true_positive_fraction: TrainingProductionMetrics,
	pub true_negative_fraction: TrainingProductionMetrics,
}

pub struct ConfusionMatrix {
	pub false_negatives: Option<u64>,
	pub true_negatives: Option<u64>,
	pub true_positives: Option<u64>,
	pub false_positives: Option<u64>,
}

pub struct TrainingProductionMetrics {
	pub production: Option<f32>,
	pub training: f32,
}

pub fn render(_props: Props, _page_info: PageInfo) -> String {
	todo!()
}
