use super::RunningMetric;
use ndarray::prelude::*;
use num_traits::ToPrimitive;

pub struct ClassificationMetrics {
	/// The shape of the confusion matrix is (n_classes x n_classes).
	confusion_matrix: Array2<u64>,
}

pub struct ClassificationMetricsInput<'a, 'b> {
	// (n_classes, n_examples)
	pub probabilities: ArrayView2<'a, f32>,
	// (n_examples), 1-indexed
	pub labels: ArrayView1<'b, usize>,
}

#[derive(Debug)]
pub struct ClassificationMetricsOutput {
	pub class_metrics: Vec<ClassMetrics>,
	pub accuracy: f32,
	pub precision_unweighted: f32,
	pub precision_weighted: f32,
	pub recall_unweighted: f32,
	pub recall_weighted: f32,
	pub baseline_accuracy: f32,
}

#[derive(Debug)]
pub struct ClassMetrics {
	pub true_positives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub false_negatives: u64,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
}

impl ClassificationMetrics {
	pub fn new(n_classes: usize) -> Self {
		//                                           prediction    label
		//                                               |           |
		//                                               v           v
		let confusion_matrix = <Array2<u64>>::zeros((n_classes, n_classes));
		Self { confusion_matrix }
	}
}

impl<'a, 'b> RunningMetric<'a, 'b> for ClassificationMetrics {
	type Input = ClassificationMetricsInput<'a, 'b>;
	type Output = ClassificationMetricsOutput;

	fn update(&mut self, value: ClassificationMetricsInput) {
		for (label, probabilities) in value.labels.iter().zip(value.probabilities.genrows()) {
			let prediction = probabilities
				.iter()
				.enumerate()
				.max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
				.unwrap()
				.0;
			// labels are 1-indexed, convert to 0-indexed
			let label = label.checked_sub(1).unwrap();
			self.confusion_matrix[(prediction, label)] += 1;
		}
	}

	fn merge(&mut self, other: Self) {
		self.confusion_matrix += &other.confusion_matrix;
	}

	fn finalize(self) -> ClassificationMetricsOutput {
		let n_classes = self.confusion_matrix.nrows();
		let n_examples = self.confusion_matrix.sum();
		let confusion_matrix = self.confusion_matrix;
		let class_metrics: Vec<_> = (0..n_classes)
			.map(|class_index| {
				let true_positives = confusion_matrix[(class_index, class_index)];
				let false_positives = confusion_matrix.row(class_index).sum() - true_positives;
				let false_negatives = confusion_matrix.column(class_index).sum() - true_positives;
				let true_negatives =
					n_examples - true_positives - false_positives - false_negatives;
				let accuracy = (true_positives + true_negatives).to_f32().unwrap()
					/ n_examples.to_f32().unwrap();
				let precision = true_positives.to_f32().unwrap()
					/ (true_positives + false_positives).to_f32().unwrap();
				let recall = true_positives.to_f32().unwrap()
					/ (true_positives + false_negatives).to_f32().unwrap();
				let f1_score = 2.0 * (precision * recall) / (precision + recall);
				ClassMetrics {
					true_positives,
					false_positives,
					true_negatives,
					false_negatives,
					accuracy,
					precision,
					recall,
					f1_score,
				}
			})
			.collect();
		let n_correct: u64 = confusion_matrix.diag().sum();
		let accuracy = n_correct.to_f32().unwrap() / n_examples.to_f32().unwrap();
		let precision_unweighted = class_metrics
			.iter()
			.map(|class| class.precision)
			.sum::<f32>()
			/ n_classes.to_f32().unwrap();
		let recall_unweighted = class_metrics.iter().map(|class| class.recall).sum::<f32>()
			/ n_classes.to_f32().unwrap();
		let n_examples_per_class = confusion_matrix.sum_axis(Axis(0));
		let precision_weighted = class_metrics
			.iter()
			.zip(n_examples_per_class.iter())
			.map(|(class, &n_examples_in_class)| {
				class.precision * n_examples_in_class.to_f32().unwrap()
			})
			.sum::<f32>()
			/ n_examples.to_f32().unwrap();
		let recall_weighted = class_metrics
			.iter()
			.zip(n_examples_per_class.iter())
			.map(|(class, &n_examples_in_class)| {
				class.recall * n_examples_in_class.to_f32().unwrap()
			})
			.sum::<f32>()
			/ n_examples.to_f32().unwrap();
		let baseline_accuracy = n_examples_per_class
			.iter()
			.map(|n| n.to_f32().unwrap())
			.fold(None, |a: Option<f32>, b| match a {
				None => Some(b),
				Some(a) => Some(a.max(b)),
			})
			.unwrap() / n_examples.to_f32().unwrap();
		ClassificationMetricsOutput {
			accuracy,
			baseline_accuracy,
			class_metrics,
			precision_unweighted,
			precision_weighted,
			recall_unweighted,
			recall_weighted,
		}
	}
}

#[test]
fn test_binary() {
	let classes = vec![String::from("Cat"), String::from("Dog")];
	let mut metrics = ClassificationMetrics::new(classes.len());
	let labels = arr1(&[1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2]);
	let probabilities = arr2(&[
		[1.0, 0.0], // correct
		[1.0, 0.0], // correct
		[1.0, 0.0], // correct
		[1.0, 0.0], // correct
		[1.0, 0.0], // correct
		[0.0, 1.0], // incorrect
		[0.0, 1.0], // incorrect
		[0.0, 1.0], // incorrect
		[0.0, 1.0], // correct
		[0.0, 1.0], // correct
		[0.0, 1.0], // correct
		[1.0, 0.0], // incorrect
		[1.0, 0.0], // incorrect
	]);
	metrics.update(ClassificationMetricsInput {
		probabilities: probabilities.view(),
		labels: labels.view(),
	});
	let metrics = metrics.finalize();
	insta::assert_debug_snapshot!(metrics, @r###"
 ClassificationMetricsOutput {
     class_metrics: [
         ClassMetrics {
             true_positives: 5,
             false_positives: 2,
             true_negatives: 3,
             false_negatives: 3,
             accuracy: 0.61538464,
             precision: 0.71428573,
             recall: 0.625,
             f1_score: 0.6666667,
         },
         ClassMetrics {
             true_positives: 3,
             false_positives: 3,
             true_negatives: 5,
             false_negatives: 2,
             accuracy: 0.61538464,
             precision: 0.5,
             recall: 0.6,
             f1_score: 0.54545456,
         },
     ],
     accuracy: 0.61538464,
     precision_unweighted: 0.60714287,
     precision_weighted: 0.6318681,
     recall_unweighted: 0.6125,
     recall_weighted: 0.61538464,
     baseline_accuracy: 0.61538464,
 }
 "###);
}

#[test]
fn test_multiclass() {
	// example taken from https://en.wikipedia.org/wiki/Confusion_matrix
	let classes = vec![
		String::from("Cat"),
		String::from("Dog"),
		String::from("Rabbit"),
	];
	let mut metrics = ClassificationMetrics::new(classes.len());
	let labels = arr1(&[
		1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 2, 2, 2, 3, 3, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
	]);
	let probabilities = arr2(&[
		[1.0, 0.0, 0.0], // correct
		[1.0, 0.0, 0.0], // correct
		[1.0, 0.0, 0.0], // correct
		[1.0, 0.0, 0.0], // correct
		[1.0, 0.0, 0.0], // correct
		[1.0, 0.0, 0.0], // incorrect
		[1.0, 0.0, 0.0], // incorrect
		[0.0, 1.0, 0.0], // incoorrect
		[0.0, 1.0, 0.0], // incorrect
		[0.0, 1.0, 0.0], // incorrect
		[0.0, 1.0, 0.0], // correct
		[0.0, 1.0, 0.0], // correct
		[0.0, 1.0, 0.0], // correct
		[0.0, 1.0, 0.0], // incorrect
		[0.0, 1.0, 0.0], // incorrect
		[0.0, 0.0, 1.0], // incorrect
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
		[0.0, 0.0, 1.0], // correct
	]);
	metrics.update(ClassificationMetricsInput {
		probabilities: probabilities.view(),
		labels: labels.view(),
	});
	let metrics = metrics.finalize();
	insta::assert_debug_snapshot!(metrics, @r###"
 ClassificationMetricsOutput {
     class_metrics: [
         ClassMetrics {
             true_positives: 5,
             false_positives: 2,
             true_negatives: 17,
             false_negatives: 3,
             accuracy: 0.8148148,
             precision: 0.71428573,
             recall: 0.625,
             f1_score: 0.6666667,
         },
         ClassMetrics {
             true_positives: 3,
             false_positives: 5,
             true_negatives: 16,
             false_negatives: 3,
             accuracy: 0.7037037,
             precision: 0.375,
             recall: 0.5,
             f1_score: 0.42857143,
         },
         ClassMetrics {
             true_positives: 11,
             false_positives: 1,
             true_negatives: 13,
             false_negatives: 2,
             accuracy: 0.8888889,
             precision: 0.9166667,
             recall: 0.84615386,
             f1_score: 0.88,
         },
     ],
     accuracy: 0.7037037,
     precision_unweighted: 0.6686508,
     precision_weighted: 0.7363316,
     recall_unweighted: 0.65705127,
     recall_weighted: 0.7037037,
     baseline_accuracy: 0.4814815,
 }
 "###);
}
