use super::StreamingMetric;
use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::s;
use num_traits::ToPrimitive;

/**
BinaryClassificationMetrics computes common metrics used to evaluate binary classifiers at various classification thresholds. Instead of computing threshold metrics for each prediction probability, we instead compute metrics for a fixed number of threshold values given by `n_thresholds` passed to [BinaryClassificationMetrics::new](struct.BinaryClassificationMetrics.html#method.new). This is an approximation but is more memory efficient.
*/
pub struct BinaryClassificationMetrics {
	/// The confusion matrices is an array of shape n_thresholds x (n_classes x n_classes).
	/// The inner `Array2<u64>` is a per-threshold [Confusion Matrix](https://en.wikipedia.org/wiki/Confusion_matrix).
	pub confusion_matrices: Array3<u64>,
	/// The thresholds are evenly-spaced between 0 and 1 based on the total number of thresholds: `n_thresholds`, passed to [BinaryClassificationMetrics::new](struct.BinaryClassificationMetrics.html#method.new).
	pub thresholds: Vec<f32>,
}

/// The input to [BinaryClassificationMetrics](struct.BinaryClassificationMetrics.html).
pub struct BinaryClassificationMetricsInput<'a> {
	pub probabilities: ArrayView2<'a, f32>,
	pub labels: ArrayView1<'a, usize>,
}

/// BinaryClassificationMetrics contains common metrics used to evaluate binary classifiers.
#[derive(Debug)]
pub struct BinaryClassificationMetricsOutput {
	/// This contains metrics specific to each class.
	pub class_metrics: Vec<BinaryClassificationClassMetricsOutput>,
}

/// BinaryClassificationClassMetricsOutput contains class specific metrics including metrics at each classification threshold.
#[derive(Debug)]
pub struct BinaryClassificationClassMetricsOutput {
	pub thresholds: Vec<BinaryClassificationThresholdMetricsOutput>,
	/// The area under the receiver operating characteristic curve is computed using a fixed number of thresholds equal to `n_thresholds` which is passed to[BinaryClassificationMetrics::new](struct.BinaryClassificationMetrics.html#method.new).
	pub auc_roc: f32,
}

/// The output from [BinaryClassificationMetrics](struct.BinaryClassificationMetrics.html).
#[derive(Debug)]
pub struct BinaryClassificationThresholdMetricsOutput {
	/// The classification threshold.
	pub threshold: f32,
	/// The total number of examples whose label is equal to this class that the model predicted as belonging to this class.
	pub true_positives: u64,
	/// The total number of examples whose label is *not* equal to this class that the model predicted as belonging to this class.
	pub false_positives: u64,
	/// The total number of examples whose label is *not* equal to this class that the model predicted as *not* belonging to this class.
	pub true_negatives: u64,
	/// The total number of examples whose label is equal to this class that the model predicted as *not* belonging to this class.
	pub false_negatives: u64,
	/// The fraction of examples of this class that were correctly classified.
	pub accuracy: f32,
	/// The precision is the fraction of examples the model predicted as belonging to this class whose label is actually equal to this class.
	/// true_positives / (true_positives + false_positives). See [Precision and Recall](https://en.wikipedia.org/wiki/Precision_and_recall).
	pub precision: f32,
	/// The recall is the fraction of examples whose label is equal to this class that the model predicted as belonging to this class.
	/// true_positives / (true_positives + false_negatives)
	pub recall: f32,
	/// The f1 score is the harmonic mean of the precision and the recall. See [F1 Score](https://en.wikipedia.org/wiki/F1_score).
	pub f1_score: f32,
	/// The true positive rate is the fraction of examples whose label is equal to this class that the model predicted as belonging to this class. Also known as the recall.
	/// See [Sensitivity and Specificity](https://en.wikipedia.org/wiki/Sensitivity_and_specificity).
	pub true_positive_rate: f32,
	/// The false positive rate is the fraction of examples whose label is not equal to this class that the model falsely predicted as belonging to this class.
	// false_positives / (false_positives + true_negatives). See [False Positive Rate](https://en.wikipedia.org/wiki/False_positive_rate)
	pub false_positive_rate: f32,
}

impl BinaryClassificationMetrics {
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
				// labels are 1-indexed
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
		let negative_class_metrics = compute_class_metrics(
			0,
			self.thresholds.as_slice(),
			self.confusion_matrices.view(),
		);
		let positive_class_metrics = compute_class_metrics(
			1,
			self.thresholds.as_slice(),
			self.confusion_matrices.view(),
		);
		let class_metrics = vec![negative_class_metrics, positive_class_metrics];
		BinaryClassificationMetricsOutput { class_metrics }
	}
}

/// Compute class metrics for each threshold.
fn compute_class_metrics(
	class_index: usize,
	thresholds: &[f32],
	confusion_matrices: ArrayView3<u64>,
) -> BinaryClassificationClassMetricsOutput {
	let thresholds: Vec<_> = thresholds
		.iter()
		.enumerate()
		.map(|(threshold_index, &threshold)| {
			let slice = s![threshold_index, .., ..];
			let confusion_matrix = confusion_matrices.slice(slice);
			/*
			class 0:
									actual
									0		1
			predicted	0	tp	fp
								1	fn	tn

			class 1:
									actual
									0		1
			predicted	0	tn	fn
								1	fp	tp
			*/
			let n_examples = confusion_matrix.sum();
			// true positives for a given class are when the predicted == actual which for the negative (0th) class this is in the 0, 0 entry and for the 1st (positive) class, this is in the 1, 1 entry
			let true_positives = confusion_matrix[(class_index, class_index)];
			// false positives are computed by taking the total predicted positives and subtracting the true positives
			let false_positives = confusion_matrix.row(class_index).sum() - true_positives;
			// false negatives are computed by taking the total actual positives and subtracting the true positives
			let false_negatives = confusion_matrix.column(class_index).sum() - true_positives;
			// true negatives are computed by subtracting false_positives, false_negatives, and true_positives from the total number of examples
			let true_negatives = n_examples - false_positives - false_negatives - true_positives;
			// the fraction of the total predictions that are correct
			let accuracy =
				(true_positives + true_negatives).to_f32().unwrap() / n_examples.to_f32().unwrap();
			// the fraction of the total predictive positive examples that are actually positive
			let precision = true_positives.to_f32().unwrap()
				/ (true_positives + false_positives).to_f32().unwrap();
			// the fraction of the total positive examples that are correctly predicted as positive
			let recall = true_positives.to_f32().unwrap()
				/ (true_positives + false_negatives).to_f32().unwrap();
			let f1_score = 2.0 * (precision * recall) / (precision + recall);
			// true_positive_rate = true_positives / positives
			let true_positive_rate = (true_positives.to_f32().unwrap())
				/ (true_positives.to_f32().unwrap() + false_negatives.to_f32().unwrap());
			// false_positive_rate = false_positives / negatives
			let false_positive_rate = false_positives.to_f32().unwrap()
				/ (true_negatives.to_f32().unwrap() + false_positives.to_f32().unwrap());
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

	// compute the area under the receiver operating characteristic curve using a riemann sum
	let auc_roc = if class_index == 0 {
		// for the negative class, the thresholds are in the correct order
		thresholds
			.iter()
			.tuple_windows()
			.map(|(left, right)| {
				// trapezoidal rule
				let y_avg = (left.true_positive_rate + right.true_positive_rate) / 2.0;
				let dx = right.false_positive_rate - left.false_positive_rate;
				y_avg * dx
			})
			.sum::<f32>()
	} else {
		// for the positive class, the thresholds are in the reverse order
		thresholds
			.iter()
			.rev()
			.tuple_windows()
			.map(|(left, right)| {
				// trapezoidal rule
				let y_avg = (left.true_positive_rate + right.true_positive_rate) / 2.0;
				let dx = right.false_positive_rate - left.false_positive_rate;
				y_avg * dx
			})
			.sum::<f32>()
	};

	BinaryClassificationClassMetricsOutput {
		thresholds,
		auc_roc,
	}
}

#[test]
fn test() {
	let mut metrics = BinaryClassificationMetrics::new(4);
	// labels are 1-indexed
	let labels = arr1(&[1, 1, 1, 1, 2, 2, 2, 2]);
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
	metrics.update(BinaryClassificationMetricsInput {
		probabilities: probabilities.view(),
		labels: labels.view(),
	});
	let metrics = metrics.finalize();
	println!("{:?}", metrics);
	insta::assert_debug_snapshot!(metrics, @r###"
 BinaryClassificationMetricsOutput {
     class_metrics: [
         BinaryClassificationClassMetricsOutput {
             thresholds: [
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.0,
                     true_positives: 0,
                     false_positives: 0,
                     true_negatives: 4,
                     false_negatives: 4,
                     accuracy: 0.5,
                     precision: NaN,
                     recall: 0.0,
                     f1_score: NaN,
                     true_positive_rate: 0.0,
                     false_positive_rate: 0.0,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.25,
                     true_positives: 0,
                     false_positives: 0,
                     true_negatives: 4,
                     false_negatives: 4,
                     accuracy: 0.5,
                     precision: NaN,
                     recall: 0.0,
                     f1_score: NaN,
                     true_positive_rate: 0.0,
                     false_positive_rate: 0.0,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.5,
                     true_positives: 3,
                     false_positives: 1,
                     true_negatives: 3,
                     false_negatives: 1,
                     accuracy: 0.75,
                     precision: 0.75,
                     recall: 0.75,
                     f1_score: 0.75,
                     true_positive_rate: 0.75,
                     false_positive_rate: 0.25,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.75,
                     true_positives: 4,
                     false_positives: 4,
                     true_negatives: 0,
                     false_negatives: 0,
                     accuracy: 0.5,
                     precision: 0.5,
                     recall: 1.0,
                     f1_score: 0.6666667,
                     true_positive_rate: 1.0,
                     false_positive_rate: 1.0,
                 },
             ],
             auc_roc: 0.75,
         },
         BinaryClassificationClassMetricsOutput {
             thresholds: [
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.0,
                     true_positives: 4,
                     false_positives: 4,
                     true_negatives: 0,
                     false_negatives: 0,
                     accuracy: 0.5,
                     precision: 0.5,
                     recall: 1.0,
                     f1_score: 0.6666667,
                     true_positive_rate: 1.0,
                     false_positive_rate: 1.0,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.25,
                     true_positives: 4,
                     false_positives: 4,
                     true_negatives: 0,
                     false_negatives: 0,
                     accuracy: 0.5,
                     precision: 0.5,
                     recall: 1.0,
                     f1_score: 0.6666667,
                     true_positive_rate: 1.0,
                     false_positive_rate: 1.0,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.5,
                     true_positives: 3,
                     false_positives: 1,
                     true_negatives: 3,
                     false_negatives: 1,
                     accuracy: 0.75,
                     precision: 0.75,
                     recall: 0.75,
                     f1_score: 0.75,
                     true_positive_rate: 0.75,
                     false_positive_rate: 0.25,
                 },
                 BinaryClassificationThresholdMetricsOutput {
                     threshold: 0.75,
                     true_positives: 0,
                     false_positives: 0,
                     true_negatives: 4,
                     false_negatives: 4,
                     accuracy: 0.5,
                     precision: NaN,
                     recall: 0.0,
                     f1_score: NaN,
                     true_positive_rate: 0.0,
                     false_positive_rate: 0.0,
                 },
             ],
             auc_roc: 0.75,
         },
     ],
 }
 "###);
}
