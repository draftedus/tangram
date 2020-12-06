#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientProps {
	pub threshold_metrics: Vec<Metrics>,
	pub baseline_metrics: Metrics,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Metrics {
	pub accuracy: Option<f32>,
	pub f1_score: Option<f32>,
	pub false_negatives: u64,
	pub false_positives: u64,
	pub precision: Option<f32>,
	pub recall: Option<f32>,
	pub threshold: f32,
	pub true_negatives: u64,
	pub true_positives: u64,
}
