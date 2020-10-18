use crate::common::monitor_event::NumberOrString;
use num_traits::ToPrimitive;
use tangram_metrics::StreamingMetric;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BinaryClassificationProductionPredictionMetrics {
	confusion_matrix: BinaryConfusionMatrix,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct BinaryConfusionMatrix {
	false_negatives: u64,
	false_positives: u64,
	true_negatives: u64,
	true_positives: u64,
}

impl BinaryConfusionMatrix {
	fn new() -> Self {
		Self {
			false_negatives: 0,
			false_positives: 0,
			true_negatives: 0,
			true_positives: 0,
		}
	}
	fn n_examples(&self) -> u64 {
		self.false_positives + self.false_negatives + self.true_positives + self.true_negatives
	}
}

#[derive(Debug)]
pub struct BinaryClassificationProductionPredictionMetricsOutput {
	pub accuracy: f32,
	pub f1_score: f32,
	pub false_negatives: u64,
	pub false_positives: u64,
	pub precision: f32,
	pub recall: f32,
	pub true_negatives: u64,
	pub true_positives: u64,
}

impl BinaryClassificationProductionPredictionMetrics {
	pub fn new() -> BinaryClassificationProductionPredictionMetrics {
		let confusion_matrix = BinaryConfusionMatrix::new();
		BinaryClassificationProductionPredictionMetrics { confusion_matrix }
	}
}

impl StreamingMetric<'_> for BinaryClassificationProductionPredictionMetrics {
	type Input = (NumberOrString, NumberOrString);
	type Output = Option<BinaryClassificationProductionPredictionMetricsOutput>;

	fn update(&mut self, value: Self::Input) {
		let label = match value.1 {
			NumberOrString::Number(_) => return,
			NumberOrString::String(s) => s,
		};
		let prediction = match value.0 {
			NumberOrString::Number(_) => return,
			NumberOrString::String(s) => s,
		};
		let confusion_matrix = &mut self.confusion_matrix;
		match (label.as_str(), prediction.as_str()) {
			("1", "0") => {
				confusion_matrix.false_negatives += 1;
			}
			("0", "1") => {
				confusion_matrix.false_positives += 1;
			}
			("0", "0") => {
				confusion_matrix.true_negatives += 1;
			}
			("1", "1") => {
				confusion_matrix.true_positives += 1;
			}
			_ => todo!(),
		}
	}

	fn merge(&mut self, other: Self) {
		self.confusion_matrix.false_negatives += other.confusion_matrix.false_negatives;
		self.confusion_matrix.false_positives += other.confusion_matrix.false_positives;
		self.confusion_matrix.true_negatives += other.confusion_matrix.true_negatives;
		self.confusion_matrix.true_positives += other.confusion_matrix.true_positives;
	}

	fn finalize(self) -> Self::Output {
		let n_examples = self.confusion_matrix.n_examples();
		let true_positives = self.confusion_matrix.true_positives;
		let false_positives = self.confusion_matrix.false_positives;
		let false_negatives = self.confusion_matrix.false_negatives;
		let true_negatives = self.confusion_matrix.true_negatives;
		// This is the fraction of the total predictions that are correct.
		let accuracy =
			(true_positives + true_negatives).to_f32().unwrap() / n_examples.to_f32().unwrap();
		// This is the fraction of the total predictive positive examples that are actually positive.
		let precision =
			true_positives.to_f32().unwrap() / (true_positives + false_positives).to_f32().unwrap();
		// This is the fraction of the total positive examples that are correctly predicted as positive.
		let recall =
			true_positives.to_f32().unwrap() / (true_positives + false_negatives).to_f32().unwrap();
		let f1_score = 2.0 * (precision * recall) / (precision + recall);
		if n_examples == 0 {
			None
		} else {
			Some(BinaryClassificationProductionPredictionMetricsOutput {
				accuracy,
				precision,
				recall,
				f1_score,
				false_negatives,
				false_positives,
				true_negatives,
				true_positives,
			})
		}
	}
}

#[test]
fn test_binary() {
	let mut metrics = BinaryClassificationProductionPredictionMetrics::new();
	metrics.update((
		NumberOrString::String("Cat".into()),
		NumberOrString::String("Cat".into()),
	));
	let labels = vec![
		"Cat", "Cat", "Cat", "Cat", "Cat", "Cat", "Cat", "Dog", "Dog", "Dog", "Dog", "Dog",
	];
	let predictions = vec![
		"Cat", "Cat", "Cat", "Cat", "Dog", "Dog", "Dog", "Dog", "Dog", "Dog", "Cat", "Cat",
	];
	for (label, prediction) in labels.into_iter().zip(predictions.into_iter()) {
		metrics.update((
			NumberOrString::String(prediction.to_owned()),
			NumberOrString::String(label.to_owned()),
		));
	}
	let metrics = metrics.finalize();
	insta::assert_debug_snapshot!(metrics, @r###""###);
}
