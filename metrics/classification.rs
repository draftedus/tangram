use super::StreamingMetric;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::num::NonZeroUsize;

/// ClassificationMetrics computes common metrics used to evaluate classifiers.
pub struct ClassificationMetrics {
	/// The shape of the confusion matrix is (n_classes x n_classes).
	confusion_matrix: Array2<u64>,
}

/// The input to [ClassificationMetrics](struct.ClassificationMetrics.html).
pub struct ClassificationMetricsInput<'a> {
	/// (n_examples, n_classes)
	pub probabilities: ArrayView2<'a, f32>,
	// (n_examples), 1-indexed
	pub labels: ArrayView1<'a, Option<NonZeroUsize>>,
}

/// The output from [ClassificationMetrics](struct.ClassificationMetrics.html).
#[derive(Debug)]
pub struct ClassificationMetricsOutput {
	/// The class metrics contain class specific metrics.
	pub class_metrics: Vec<ClassMetrics>,
	/// The accuracy is the fraction of all of the predictions that are correct.
	pub accuracy: f32,
	/// The unweighted precision equal to the mean of each class's precision.
	pub precision_unweighted: f32,
	/// The weighted precision is a weighted mean of each class's precision weighted by the fraction of the total examples in the class.
	pub precision_weighted: f32,
	/// The unweighted recall equal to the mean of each class's recall.
	pub recall_unweighted: f32,
	/// The weighted recall is a weighted mean of each class's recall weighted by the fraction of the total examples in the class.
	pub recall_weighted: f32,
	/// The accuracy if the model always predicted the majority class.
	pub baseline_accuracy: f32,
}

/// ClassMetrics are class specific metrics used to evaluate the model's performance on each individual class.
#[derive(Debug)]
pub struct ClassMetrics {
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
	/// The recall is the fraction of examples in the dataset whose label is equal to this class that the model predicted as equal to this class.
	/// true_positives / (true_positives + false_negatives)
	pub recall: f32,
	/// The f1 score is the harmonic mean of the precision and the recall. See [F1 Score](https://en.wikipedia.org/wiki/F1_score).
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

impl<'a> StreamingMetric<'a> for ClassificationMetrics {
	type Input = ClassificationMetricsInput<'a>;
	type Output = ClassificationMetricsOutput;

	fn update(&mut self, value: ClassificationMetricsInput) {
		for (label, probabilities) in value.labels.iter().zip(value.probabilities.genrows()) {
			let prediction = probabilities
				.iter()
				.enumerate()
				.max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
				.unwrap()
				.0;
			// get index in confusion matrix for label
			let label = label.unwrap().get() - 1;
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
	let labels = arr1(&[
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
	]);
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
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
		Some(NonZeroUsize::new(3).unwrap()),
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
