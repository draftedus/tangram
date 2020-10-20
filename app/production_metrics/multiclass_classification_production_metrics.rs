use crate::common::monitor_event::NumberOrString;
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use tangram_metrics::StreamingMetric;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct MulticlassClassificationProductionPredictionMetrics {
	classes: Vec<String>,
	confusion_matrix: Array2<u64>,
}

#[derive(Debug)]
pub struct MulticlassClassificationProductionPredictionMetricsOutput {
	pub class_metrics: Vec<MulticlassClassificationProductionPredictionClassMetricsOutput>,
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub precision_unweighted: f32,
	pub precision_weighted: f32,
	pub recall_unweighted: f32,
	pub recall_weighted: f32,
}

#[derive(Debug)]
pub struct MulticlassClassificationProductionPredictionClassMetricsOutput {
	pub class_name: String,
	pub true_positives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub false_negatives: u64,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
}

impl MulticlassClassificationProductionPredictionMetrics {
	pub fn new(classes: Vec<String>) -> MulticlassClassificationProductionPredictionMetrics {
		let n_classes = classes.len();
		let confusion_matrix = <Array2<u64>>::zeros((n_classes, n_classes));
		MulticlassClassificationProductionPredictionMetrics {
			classes,
			confusion_matrix,
		}
	}
}

impl StreamingMetric<'_> for MulticlassClassificationProductionPredictionMetrics {
	type Input = (NumberOrString, NumberOrString);
	type Output = Option<MulticlassClassificationProductionPredictionMetricsOutput>;

	fn update(&mut self, value: Self::Input) {
		let label = match value.1 {
			NumberOrString::Number(_) => return,
			NumberOrString::String(s) => s,
		};
		let prediction = match value.0 {
			NumberOrString::Number(_) => return,
			NumberOrString::String(s) => s,
		};
		let actual_label_id = match self.classes.iter().position(|c| *c == label) {
			Some(position) => position,
			None => return,
		};
		if let Some(predicted_label_id) = self.classes.iter().position(|c| *c == prediction) {
			self.confusion_matrix[(predicted_label_id, actual_label_id)] += 1
		}
	}

	fn merge(&mut self, other: Self) {
		self.confusion_matrix += &other.confusion_matrix;
	}

	fn finalize(self) -> Self::Output {
		let n_classes = self.classes.len();
		let n_examples = self.confusion_matrix.sum();
		let confusion_matrix = self.confusion_matrix;
		let class_metrics: Vec<_> = self
			.classes
			.into_iter()
			.enumerate()
			.map(|(class_index, class_name)| {
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
				MulticlassClassificationProductionPredictionClassMetricsOutput {
					class_name,
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
		let precision_weighted = izip!(&class_metrics, &n_examples_per_class)
			.map(|(class, n_examples_in_class)| {
				class.precision * n_examples_in_class.to_f32().unwrap()
			})
			.sum::<f32>()
			/ n_examples.to_f32().unwrap();
		let recall_weighted = izip!(&class_metrics, &n_examples_per_class)
			.map(|(class, n_examples_in_class)| {
				class.recall * n_examples_in_class.to_f32().unwrap()
			})
			.sum::<f32>()
			/ n_examples.to_f32().unwrap();
		let baseline_accuracy = n_examples_per_class
			.iter()
			.map(|n| n.to_f32().unwrap())
			.fold(None, |a: Option<f32>, b| match a {
				None => Some(b),
				Some(a) => Some(f32::max(a, b)),
			})
			.unwrap() / n_examples.to_f32().unwrap();
		if n_examples == 0 {
			None
		} else {
			Some(MulticlassClassificationProductionPredictionMetricsOutput {
				accuracy,
				baseline_accuracy,
				class_metrics,
				precision_unweighted,
				precision_weighted,
				recall_unweighted,
				recall_weighted,
			})
		}
	}
}

#[test]
fn test_binary() {
	let classes = vec!["Cat".into(), "Dog".into()];
	let mut metrics = MulticlassClassificationProductionPredictionMetrics::new(classes);
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
	for (label, prediction) in izip!(labels, predictions) {
		metrics.update((
			NumberOrString::String(prediction.to_owned()),
			NumberOrString::String(label.to_owned()),
		));
	}
	let metrics = metrics.finalize();
	insta::assert_debug_snapshot!(metrics, @r###"
 Some(
     MulticlassClassificationProductionPredictionMetricsOutput {
         class_metrics: [
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Cat",
                 true_positives: 5,
                 false_positives: 2,
                 true_negatives: 3,
                 false_negatives: 3,
                 accuracy: 0.61538464,
                 precision: 0.71428573,
                 recall: 0.625,
                 f1_score: 0.6666667,
             },
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Dog",
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
         baseline_accuracy: 0.61538464,
         precision_unweighted: 0.60714287,
         precision_weighted: 0.6318681,
         recall_unweighted: 0.6125,
         recall_weighted: 0.61538464,
     },
 )
 "###);
}

#[test]
fn test_multiclass() {
	// This example is taken from https://en.wikipedia.org/wiki/Confusion_matrix.
	let classes = vec!["Cat".into(), "Dog".into(), "Rabbit".into()];
	let mut metrics = MulticlassClassificationProductionPredictionMetrics::new(classes);
	metrics.update((
		NumberOrString::String("Cat".into()),
		NumberOrString::String("Cat".into()),
	));
	let labels = vec![
		"Cat", "Cat", "Cat", "Cat", "Dog", "Dog", "Cat", "Cat", "Cat", "Dog", "Dog", "Dog",
		"Rabbit", "Rabbit", "Dog", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit",
		"Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit",
	];
	let predictions = vec![
		"Cat", "Cat", "Cat", "Cat", "Cat", "Cat", "Dog", "Dog", "Dog", "Dog", "Dog", "Dog", "Dog",
		"Dog", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit",
		"Rabbit", "Rabbit", "Rabbit", "Rabbit",
	];
	for (label, prediction) in izip!(labels, predictions) {
		metrics.update((
			NumberOrString::String(prediction.to_owned()),
			NumberOrString::String(label.to_owned()),
		));
	}
	let metrics = metrics.finalize();
	insta::assert_debug_snapshot!(metrics, @r###"
 Some(
     MulticlassClassificationProductionPredictionMetricsOutput {
         class_metrics: [
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Cat",
                 true_positives: 5,
                 false_positives: 2,
                 true_negatives: 17,
                 false_negatives: 3,
                 accuracy: 0.8148148,
                 precision: 0.71428573,
                 recall: 0.625,
                 f1_score: 0.6666667,
             },
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Dog",
                 true_positives: 3,
                 false_positives: 5,
                 true_negatives: 16,
                 false_negatives: 3,
                 accuracy: 0.7037037,
                 precision: 0.375,
                 recall: 0.5,
                 f1_score: 0.42857143,
             },
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Rabbit",
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
         baseline_accuracy: 0.4814815,
         precision_unweighted: 0.6686508,
         precision_weighted: 0.7363316,
         recall_unweighted: 0.65705127,
         recall_weighted: 0.7037037,
     },
 )
 "###);
}
