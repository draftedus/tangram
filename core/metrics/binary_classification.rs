use super::RunningMetric;
use ndarray::prelude::*;
use ndarray::s;
use num_traits::ToPrimitive;

pub struct BinaryClassifierMetrics {
	/// The shape of the confusion matrix is
	/// thresholds x (n_classes x n_classes).
	/// The two lower indexes are a confusion matrix.
	pub confusion_matrices: Array3<u64>,
	pub thresholds: Vec<f32>,
}

pub struct BinaryClassifierMetricsInput<'a, 'b> {
	pub probabilities: ArrayView2<'a, f32>,
	pub labels: ArrayView1<'b, usize>,
}

#[derive(Debug)]
pub struct BinaryClassificationMetricsOutput {
	pub class_metrics: Vec<BinaryClassificationClassMetricsOutput>,
	pub auc_roc: f32,
}

#[derive(Debug)]
pub struct BinaryClassificationClassMetricsOutput {
	pub thresholds: Vec<BinaryClassificationThresholdMetricsOutput>,
}

#[derive(Debug)]
pub struct BinaryClassificationThresholdMetricsOutput {
	pub threshold: f32,
	pub true_positives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub false_negatives: u64,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
	pub true_positive_rate: f32,
	pub false_positive_rate: f32,
}

impl BinaryClassifierMetrics {
	pub fn new(n_thresholds: usize) -> Self {
		let thresholds = (0..n_thresholds)
			.map(|i| i.to_f32().unwrap() * (1.0 / n_thresholds.to_f32().unwrap()))
			.collect();
		let n_classes = 2;
		//            threshold_index  prediction  label
		//                  |           |          /
		//                  v           v         v
		let shape = (n_thresholds, n_classes, n_classes);
		Self {
			confusion_matrices: Array3::zeros(shape),
			thresholds,
		}
	}
}

impl<'a, 'b> RunningMetric<'a, 'b> for BinaryClassifierMetrics {
	type Input = BinaryClassifierMetricsInput<'a, 'b>;
	type Output = BinaryClassificationMetricsOutput;

	fn update(&mut self, value: BinaryClassifierMetricsInput) {
		let n_examples = value.labels.len();
		for (threshold_index, &threshold) in self.thresholds.iter().enumerate() {
			for example_index in 0..n_examples {
				let predicted_label_id = if value.probabilities[(example_index, 1)] > threshold {
					1
				} else {
					0
				};
				let actual_label_id = if value.labels[example_index] == 2 {
					1
				} else {
					0
				};
				let position = (threshold_index, predicted_label_id, actual_label_id);
				self.confusion_matrices[position] += 1;
			}
		}
	}

	fn merge(&mut self, other: Self) {
		self.confusion_matrices += &other.confusion_matrices;
	}

	fn finalize(self) -> BinaryClassificationMetricsOutput {
		let class_metrics = [0usize, 1]
			.iter()
			.map(|&class_index| {
				let thresholds: Vec<_> = self
					.thresholds
					.iter()
					.enumerate()
					.map(|(threshold_index, &threshold)| {
						let slice = s![threshold_index, .., ..];
						let confusion_matrix = self.confusion_matrices.slice(slice);
						let n_examples = confusion_matrix.sum();
						let true_positives = confusion_matrix[(class_index, class_index)];
						let false_positives =
							confusion_matrix.row(class_index).sum() - true_positives;
						let false_negatives =
							confusion_matrix.column(class_index).sum() - true_positives;
						let true_negatives =
							n_examples - false_positives - false_negatives - true_positives;
						let accuracy = (true_positives + true_negatives).to_f32().unwrap()
							/ n_examples.to_f32().unwrap();
						let precision = true_positives.to_f32().unwrap()
							/ (true_positives + false_positives).to_f32().unwrap();
						let recall = true_positives.to_f32().unwrap()
							/ (true_positives + false_negatives).to_f32().unwrap();
						let f1_score = 2.0 * (precision * recall) / (precision + recall);
						// tpr = tp / p = tp / (tp + fn)
						let true_positive_rate = (true_positives.to_f32().unwrap())
							/ (true_positives.to_f32().unwrap()
								+ false_negatives.to_f32().unwrap());
						// fpr = fp / n = fp / (fp + tn)
						let false_positive_rate = false_positives.to_f32().unwrap()
							/ (true_negatives.to_f32().unwrap()
								+ false_positives.to_f32().unwrap());
						BinaryClassificationThresholdMetricsOutput {
							threshold,
							false_negatives,
							false_positives,
							true_negatives,
							true_positives,
							accuracy,
							precision,
							recall,
							f1_score,
							false_positive_rate,
							true_positive_rate,
						}
					})
					.collect();
				BinaryClassificationClassMetricsOutput { thresholds }
			})
			.collect();
		let auc_roc = auc_roc(self.confusion_matrices.view());
		BinaryClassificationMetricsOutput {
			class_metrics,
			auc_roc,
		}
	}
}

// computes the auc using a riemann sum given a confusion matrix
// with a predefined number of thresholds
//
//                threshold_index  prediction   label
//                      |             |           |
//                      v             v           v
// let dimension = (n_thresholds, n_classes, n_classes);
// confusion_matrix: Array3::zeros(dimension),
fn auc_roc(confusion_matrix: ArrayView3<u64>) -> f32 {
	let class_index = 1;
	let n_thresholds = confusion_matrix.shape()[0];
	let roc_curve = (0..n_thresholds)
		.map(|threshold_index| {
			let slice = s![threshold_index, .., ..];
			let confusion_matrix = confusion_matrix.slice(slice);
			let n_examples: u64 = confusion_matrix.sum();
			let true_positives: u64 = confusion_matrix[(class_index, class_index)];
			let false_positives: u64 = confusion_matrix.row(class_index).sum() - true_positives;
			let false_negatives: u64 = confusion_matrix.column(class_index).sum() - true_positives;
			let true_negatives: u64 =
				n_examples - false_negatives - false_positives - true_positives;
			let false_positive_rate = false_positives.to_f32().unwrap()
				/ (false_positives + true_negatives).to_f32().unwrap();
			let true_positive_rate = true_positives.to_f32().unwrap()
				/ (true_positives + false_negatives).to_f32().unwrap();
			(false_positive_rate, true_positive_rate)
		})
		.collect::<Vec<(f32, f32)>>();
	(0..roc_curve.len() - 1)
		.map(|i| {
			let left = &roc_curve[i + 1];
			let right = &roc_curve[i];
			let y_left = left.1;
			let y_right = right.1;
			let y_average = (y_left + y_right) / 2.0;
			let dx = right.0 - left.0;
			y_average * dx
		})
		.sum()
}

#[test]
fn test() {
	let mut metrics = BinaryClassifierMetrics::new(4);
	let labels = arr1(&[0, 0, 0, 0, 1, 1, 1, 1]);
	let probabilities = arr2(&[
		[0.6, 0.4],
		[0.6, 0.4],
		[0.6, 0.4],
		[0.4, 0.6],
		[0.4, 0.6],
		[0.4, 0.6],
		[0.4, 0.6],
		[0.6, 0.4],
	]);
	metrics.update(BinaryClassifierMetricsInput {
		probabilities: probabilities.view(),
		labels: labels.view(),
	});
	let metrics = metrics.finalize();
	insta::assert_debug_snapshot!(metrics, @r###"
 BinaryClassificationMetricsOutput {
     class_metrics: [
         BinaryClassificationClassMetricsOutput {
             thresholds: [
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.0,
                     true_positives: 0,
                     false_positives: 0,
                     true_negatives: 0,
                     false_negatives: 8,
                     accuracy: 0.0,
                     precision: NaN,
                     recall: 0.0,
                     f1_score: NaN,
                     true_positive_rate: 0.0,
                     false_positive_rate: NaN,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.25,
                     true_positives: 0,
                     false_positives: 0,
                     true_negatives: 0,
                     false_negatives: 8,
                     accuracy: 0.0,
                     precision: NaN,
                     recall: 0.0,
                     f1_score: NaN,
                     true_positive_rate: 0.0,
                     false_positive_rate: NaN,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.5,
                     true_positives: 4,
                     false_positives: 0,
                     true_negatives: 0,
                     false_negatives: 4,
                     accuracy: 0.5,
                     precision: 1.0,
                     recall: 0.5,
                     f1_score: 0.6666667,
                     true_positive_rate: 0.5,
                     false_positive_rate: NaN,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.75,
                     true_positives: 8,
                     false_positives: 0,
                     true_negatives: 0,
                     false_negatives: 0,
                     accuracy: 1.0,
                     precision: 1.0,
                     recall: 1.0,
                     f1_score: 1.0,
                     true_positive_rate: 1.0,
                     false_positive_rate: NaN,
                 },
             ],
         },
         BinaryClassificationClassMetricsOutput {
             thresholds: [
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.0,
                     true_positives: 0,
                     false_positives: 8,
                     true_negatives: 0,
                     false_negatives: 0,
                     accuracy: 0.0,
                     precision: 0.0,
                     recall: NaN,
                     f1_score: NaN,
                     true_positive_rate: NaN,
                     false_positive_rate: 1.0,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.25,
                     true_positives: 0,
                     false_positives: 8,
                     true_negatives: 0,
                     false_negatives: 0,
                     accuracy: 0.0,
                     precision: 0.0,
                     recall: NaN,
                     f1_score: NaN,
                     true_positive_rate: NaN,
                     false_positive_rate: 1.0,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.5,
                     true_positives: 0,
                     false_positives: 4,
                     true_negatives: 4,
                     false_negatives: 0,
                     accuracy: 0.5,
                     precision: 0.0,
                     recall: NaN,
                     f1_score: NaN,
                     true_positive_rate: NaN,
                     false_positive_rate: 0.5,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.75,
                     true_positives: 0,
                     false_positives: 0,
                     true_negatives: 8,
                     false_negatives: 0,
                     accuracy: 1.0,
                     precision: NaN,
                     recall: NaN,
                     f1_score: NaN,
                     true_positive_rate: NaN,
                     false_positive_rate: 0.0,
                 },
             ],
         },
     ],
     auc_roc: NaN,
 }
 "###);
}
