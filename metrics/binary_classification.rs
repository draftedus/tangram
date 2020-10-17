use super::StreamingMetric;
use itertools::Itertools;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::num::NonZeroUsize;

/// `BinaryClassificationMetrics` computes common metrics used to evaluate binary classifiers at various classification thresholds. Instead of computing threshold metrics for each prediction probability, we instead compute metrics for a fixed number of threshold values given by `n_thresholds` passed to [BinaryClassificationMetrics::new](struct.BinaryClassificationMetrics.html#method.new). This is an approximation but is more memory efficient.
pub struct BinaryClassificationMetrics {
	/// There is one confusion matrix per threshold.
	confusion_matrices: Vec<BinaryConfusionMatrix>,
	/// The confusion matrix for the default threshold of 0.5.
	default_threshold_confusion_matrix: BinaryConfusionMatrix,
	/// The thresholds are evenly-spaced between 0 and 1 based on the total number of thresholds: `n_thresholds`, passed to [BinaryClassificationMetrics::new](struct.BinaryClassificationMetrics.html#method.new).
	thresholds: Vec<f32>,
}

#[derive(Clone)]
struct BinaryConfusionMatrix {
	false_negatives: u64,
	false_positives: u64,
	true_negatives: u64,
	true_positives: u64,
}

impl BinaryConfusionMatrix {
	fn new() -> Self {
		Self {
			false_positives: 0,
			false_negatives: 0,
			true_negatives: 0,
			true_positives: 0,
		}
	}
	fn n_examples(&self) -> u64 {
		self.false_negatives + self.false_positives + self.true_negatives + self.true_positives
	}
	fn n_correct(&self) -> u64 {
		self.true_positives + self.true_negatives
	}
}

/// The input to [BinaryClassificationMetrics](struct.BinaryClassificationMetrics.html).
pub struct BinaryClassificationMetricsInput<'a> {
	pub probabilities: ArrayView2<'a, f32>,
	pub labels: ArrayView1<'a, Option<NonZeroUsize>>,
}

/// BinaryClassificationMetrics contains common metrics used to evaluate binary classifiers.
#[derive(Debug)]
pub struct BinaryClassificationMetricsOutput {
	/// This contains metrics specific to each classification threshold.
	pub thresholds: Vec<BinaryClassificationMetricsOutputForThreshold>,
	/// The area under the receiver operating characteristic curve is computed using a fixed number of thresholds equal to `n_thresholds` which is passed to[BinaryClassificationMetrics::new](struct.BinaryClassificationMetrics.html#method.new).
	pub auc_roc: f32,
	/// The accuracy is the fraction of all of the predictions that are correct.
	pub accuracy: f32,
	/// The precision is the fraction of examples the model predicted as belonging to this class whose label is actually equal to this class. `precision = true_positives / (true_positives + false_positives)`. See [Precision and Recall](https://en.wikipedia.org/wiki/Precision_and_recall).
	pub precision: f32,
	/// The recall is the fraction of examples in the dataset whose label is equal to this class that the model predicted as equal to this class. `recall = true_positives / (true_positives + false_negatives)`.
	pub recall: f32,
}

/// The output from [BinaryClassificationMetrics](struct.BinaryClassificationMetrics.html).
#[derive(Debug)]
pub struct BinaryClassificationMetricsOutputForThreshold {
	/// The classification threshold.
	pub threshold: f32,
	/// The total number of examples whose label is equal to the positive class that the model predicted as belonging to the positive class.
	pub true_positives: u64,
	/// The total number of examples whose label is equal to the negative class that the model predicted as belonging to the positive class.
	pub false_positives: u64,
	/// The total number of examples whose label is equal to the negative class that the model predicted as belonging to the negative class.
	pub true_negatives: u64,
	/// The total number of examples whose label is equal to the positive class that the model predicted as belonging to the negative class.
	pub false_negatives: u64,
	/// The fraction of examples that were correctly classified.
	pub accuracy: f32,
	/// The precision is the fraction of examples the model predicted as belonging to the positive class whose label is actually the positive class. true_positives / (true_positives + false_positives). See [Precision and Recall](https://en.wikipedia.org/wiki/Precision_and_recall).
	pub precision: f32,
	/// The recall is the fraction of examples whose label is equal to the positive class that the model predicted as belonging to the positive class. `recall = true_positives / (true_positives + false_negatives)`.
	pub recall: f32,
	/// The f1 score is the harmonic mean of the precision and the recall. See [F1 Score](https://en.wikipedia.org/wiki/F1_score).
	pub f1_score: f32,
	/// The true positive rate is the fraction of examples whose label is equal to the positive class that the model predicted as belonging to the positive class. Also known as the recall. See [Sensitivity and Specificity](https://en.wikipedia.org/wiki/Sensitivity_and_specificity).
	pub true_positive_rate: f32,
	/// The false positive rate is the fraction of examples whose label is equal to the negative class that the model falsely predicted as belonging to the positive class. false_positives / (false_positives + true_negatives). See [False Positive Rate](https://en.wikipedia.org/wiki/False_positive_rate)
	pub false_positive_rate: f32,
}

impl BinaryClassificationMetrics {
	pub fn new(n_thresholds: usize) -> BinaryClassificationMetrics {
		let thresholds = (0..n_thresholds)
			.map(|i| i.to_f32().unwrap() * (1.0 / (n_thresholds.to_f32().unwrap() - 1.0)))
			.collect();
		BinaryClassificationMetrics {
			confusion_matrices: vec![BinaryConfusionMatrix::new(); n_thresholds + 1],
			default_threshold_confusion_matrix: BinaryConfusionMatrix::new(),
			thresholds,
		}
	}
}

impl<'a> StreamingMetric<'a> for BinaryClassificationMetrics {
	type Input = BinaryClassificationMetricsInput<'a>;
	type Output = BinaryClassificationMetricsOutput;

	fn update(&mut self, value: BinaryClassificationMetricsInput) {
		let n_examples = value.labels.len();
		for (threshold_index, &threshold) in self.thresholds.iter().enumerate() {
			for example_index in 0..n_examples {
				let predicted_label_id = if value.probabilities[(example_index, 1)] >= threshold {
					1
				} else {
					0
				};
				let actual_label_id = if value.labels[example_index].unwrap().get() == 2 {
					1
				} else {
					0
				};
				let confusion_matrix_for_threshold = &mut self.confusion_matrices[threshold_index];
				match (predicted_label_id, actual_label_id) {
					(0, 0) => confusion_matrix_for_threshold.true_negatives += 1,
					(1, 1) => confusion_matrix_for_threshold.true_positives += 1,
					(1, 0) => confusion_matrix_for_threshold.false_positives += 1,
					(0, 1) => confusion_matrix_for_threshold.false_negatives += 1,
					_ => panic!(),
				};
			}
		}
		for (label, probabilities) in value
			.labels
			.iter()
			.zip(value.probabilities.axis_iter(Axis(0)))
		{
			let probability = probabilities[1];
			let predicted_label_id = if probability >= 0.5 { 1 } else { 0 };
			let actual_label_id = label.unwrap().get() - 1;
			match (predicted_label_id, actual_label_id) {
				(0, 0) => self.default_threshold_confusion_matrix.true_negatives += 1,
				(1, 1) => self.default_threshold_confusion_matrix.true_positives += 1,
				(1, 0) => self.default_threshold_confusion_matrix.false_positives += 1,
				(0, 1) => self.default_threshold_confusion_matrix.false_negatives += 1,
				_ => panic!(),
			};
		}
	}

	fn merge(&mut self, other: Self) {
		self.confusion_matrices
			.iter_mut()
			.zip(other.confusion_matrices)
			.for_each(|(confusion_matrix_a, confusion_matrix_b)| {
				confusion_matrix_a.true_positives += confusion_matrix_b.true_positives;
				confusion_matrix_a.false_negatives += confusion_matrix_b.false_negatives;
				confusion_matrix_a.true_negatives += confusion_matrix_b.true_negatives;
				confusion_matrix_a.false_positives += confusion_matrix_b.false_positives;
			});
	}

	fn finalize(self) -> BinaryClassificationMetricsOutput {
		let thresholds: Vec<_> = self
			.thresholds
			.iter()
			.enumerate()
			.map(|(threshold_index, &threshold)| {
				let confusion_matrix = &self.confusion_matrices[threshold_index];
				let n_examples = confusion_matrix.n_examples();
				let true_positives = confusion_matrix.true_positives;
				let false_positives = confusion_matrix.false_positives;
				let false_negatives = confusion_matrix.false_negatives;
				let true_negatives = confusion_matrix.true_negatives;
				// This is the fraction of the total predictions that are correct.
				let accuracy = (true_positives + true_negatives).to_f32().unwrap()
					/ n_examples.to_f32().unwrap();
				// This is the fraction of the total predictive positive examples that are actually positive.
				let precision = true_positives.to_f32().unwrap()
					/ (true_positives + false_positives).to_f32().unwrap();
				// This is the fraction of the total positive examples that are correctly predicted as positive.
				let recall = true_positives.to_f32().unwrap()
					/ (true_positives + false_negatives).to_f32().unwrap();
				let f1_score = 2.0 * (precision * recall) / (precision + recall);
				// This is true_positive_rate = true_positives / positives.
				let true_positive_rate = (true_positives.to_f32().unwrap())
					/ (true_positives.to_f32().unwrap() + false_negatives.to_f32().unwrap());
				// This is false_positive_rate = false_positives / negatives.
				let false_positive_rate = false_positives.to_f32().unwrap()
					/ (true_negatives.to_f32().unwrap() + false_positives.to_f32().unwrap());
				BinaryClassificationMetricsOutputForThreshold {
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
		// Compute the area under the receiver operating characteristic curve using a riemann sum.
		let auc_roc = thresholds
			.iter()
			.rev()
			.tuple_windows()
			.map(|(left, right)| {
				// Use the trapezoid rule.
				let y_avg = (left.true_positive_rate + right.true_positive_rate) / 2.0;
				let dx = right.false_positive_rate - left.false_positive_rate;
				y_avg * dx
			})
			.sum::<f32>();
		let n_correct = self.default_threshold_confusion_matrix.n_correct();
		let n_examples = self.default_threshold_confusion_matrix.n_examples();
		let accuracy = n_correct.to_f32().unwrap() / n_examples.to_f32().unwrap();
		let precision = self
			.default_threshold_confusion_matrix
			.true_positives
			.to_f32()
			.unwrap() / (self
			.default_threshold_confusion_matrix
			.true_positives
			.to_f32()
			.unwrap() + self
			.default_threshold_confusion_matrix
			.false_positives
			.to_f32()
			.unwrap());
		let recall = self
			.default_threshold_confusion_matrix
			.true_positives
			.to_f32()
			.unwrap() / (self
			.default_threshold_confusion_matrix
			.true_positives
			.to_f32()
			.unwrap() + self
			.default_threshold_confusion_matrix
			.false_negatives
			.to_f32()
			.unwrap());
		BinaryClassificationMetricsOutput {
			thresholds,
			auc_roc,
			accuracy,
			precision,
			recall,
		}
	}
}

#[test]
fn test() {
	let mut metrics = BinaryClassificationMetrics::new(8);
	let labels = arr1(&[
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
	]);
	let probabilities = arr2(&[[0.9, 0.1], [0.2, 0.2], [0.7, 0.3], [0.2, 0.8], [0.1, 0.9]]);
	metrics.update(BinaryClassificationMetricsInput {
		probabilities: probabilities.view(),
		labels: labels.view(),
	});
	let metrics = metrics.finalize();
	insta::assert_debug_snapshot!(metrics, @r###"
 BinaryClassificationMetricsOutput {
     thresholds: [
         BinaryClassificationThresholdMetricsOutput {
             threshold: 0.0,
             true_positives: 2,
             false_positives: 3,
             true_negatives: 0,
             false_negatives: 0,
             accuracy: 0.4,
             precision: 0.4,
             recall: 1.0,
             f1_score: 0.5714286,
             true_positive_rate: 1.0,
             false_positive_rate: 1.0,
         },
         BinaryClassificationThresholdMetricsOutput {
             threshold: 0.14285715,
             true_positives: 2,
             false_positives: 2,
             true_negatives: 1,
             false_negatives: 0,
             accuracy: 0.6,
             precision: 0.5,
             recall: 1.0,
             f1_score: 0.6666667,
             true_positive_rate: 1.0,
             false_positive_rate: 0.6666667,
         },
         BinaryClassificationThresholdMetricsOutput {
             threshold: 0.2857143,
             true_positives: 2,
             false_positives: 1,
             true_negatives: 2,
             false_negatives: 0,
             accuracy: 0.8,
             precision: 0.6666667,
             recall: 1.0,
             f1_score: 0.8,
             true_positive_rate: 1.0,
             false_positive_rate: 0.33333334,
         },
         BinaryClassificationThresholdMetricsOutput {
             threshold: 0.42857146,
             true_positives: 1,
             false_positives: 1,
             true_negatives: 2,
             false_negatives: 1,
             accuracy: 0.6,
             precision: 0.5,
             recall: 0.5,
             f1_score: 0.5,
             true_positive_rate: 0.5,
             false_positive_rate: 0.33333334,
         },
         BinaryClassificationThresholdMetricsOutput {
             threshold: 0.5714286,
             true_positives: 1,
             false_positives: 1,
             true_negatives: 2,
             false_negatives: 1,
             accuracy: 0.6,
             precision: 0.5,
             recall: 0.5,
             f1_score: 0.5,
             true_positive_rate: 0.5,
             false_positive_rate: 0.33333334,
         },
         BinaryClassificationThresholdMetricsOutput {
             threshold: 0.71428573,
             true_positives: 1,
             false_positives: 1,
             true_negatives: 2,
             false_negatives: 1,
             accuracy: 0.6,
             precision: 0.5,
             recall: 0.5,
             f1_score: 0.5,
             true_positive_rate: 0.5,
             false_positive_rate: 0.33333334,
         },
         BinaryClassificationThresholdMetricsOutput {
             threshold: 0.8571429,
             true_positives: 1,
             false_positives: 0,
             true_negatives: 3,
             false_negatives: 1,
             accuracy: 0.8,
             precision: 1.0,
             recall: 0.5,
             f1_score: 0.6666667,
             true_positive_rate: 0.5,
             false_positive_rate: 0.0,
         },
         BinaryClassificationThresholdMetricsOutput {
             threshold: 1.0,
             true_positives: 0,
             false_positives: 0,
             true_negatives: 3,
             false_negatives: 2,
             accuracy: 0.6,
             precision: NaN,
             recall: 0.0,
             f1_score: NaN,
             true_positive_rate: 0.0,
             false_positive_rate: 0.0,
         },
     ],
     auc_roc: 0.8333333,
 }
 "###);
}
