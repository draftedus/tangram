use super::Metric;
use itertools::Itertools;

/// This function computes the ROC curve. The ROC curve plot the false positive rate on the x axis and the true positive rate on the y axis for various classification thresholds.
struct AUCROC;

impl<'a> Metric<'a> for AUCROC {
	type Input = (&'a [f32], &'a [usize]);
	type Output = f32;
	fn compute(input: Self::Input) -> Self::Output {
		let (probabilities, labels) = input;
		// collect probabilities and labels into a vec of tuples
		let mut probabilities_labels: Vec<(f32, usize)> = probabilities
			.iter()
			.zip(labels.iter())
			.map(|(a, b)| (*a, *b))
			.collect();
		// sort by probabilities in descending order
		probabilities_labels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
		probabilities_labels.reverse();
		let mut true_positives_false_positives: Vec<TpsFpsPoint> = Vec::new();
		for (probability, label) in probabilities_labels.iter() {
			// labels are 1-indexed
			let label = label.checked_sub(1).unwrap();
			// if the classification threshold were to be this probability and the label is 1, the prediction is a true_positive. If the label is 0, its not a true_positive.
			let true_positive = label;
			// if the classification threshold were to be this probability and the label is 0, the prediction is a false_positive. If the label is 1, its not a false_positive.
			let false_positive = 1 - label;
			match true_positives_false_positives.last() {
				Some(last_point)
					if f32::abs(probability - last_point.probability) < std::f32::EPSILON =>
				{
					let last = true_positives_false_positives.last_mut().unwrap();
					last.true_positives += true_positive;
					last.false_positives += false_positive;
				}
				_ => {
					true_positives_false_positives.push(TpsFpsPoint {
						probability: *probability,
						true_positives: true_positive,
						false_positives: false_positive,
					});
				}
			}
		}
		for i in 1..true_positives_false_positives.len() {
			true_positives_false_positives[i].true_positives +=
				true_positives_false_positives[i - 1].true_positives;
			true_positives_false_positives[i].false_positives +=
				true_positives_false_positives[i - 1].false_positives;
		}
		// get the total count of positives
		let count_positives = labels
			.iter()
			.map(|l| l.checked_sub(1).unwrap())
			.sum::<usize>();
		// get the total count of negatives
		let count_negatives = labels.len() - count_positives;
		// The true_positive_rate at threshold x is the percent of the total positives that have a prediction probability >= x. At the maximum probability `x` observed in the dataset, either the true_positive_rate or false_positive_rate will be nonzero depending on whether the label at the this highest probability point is positive or negative respectively. This means that we will not have a point on the ROC curve with a true_positive_rate and false_positive_rate of 0. We create a dummy point with an impossible threshold of 1.1 such that no predictions have probability >= 1.1. At this threshold, both the true_positive_rate and false_positive_rate is 0.
		let mut roc_curve = vec![ROCCurvePoint {
			threshold: 1.1,
			true_positive_rate: 0.0,
			false_positive_rate: 0.0,
		}];
		for tps_fps_point in true_positives_false_positives.iter() {
			roc_curve.push(ROCCurvePoint {
				// The true positive rate is the number of true_positives divided by the total number of positives
				true_positive_rate: tps_fps_point.true_positives as f32 / count_positives as f32,
				threshold: tps_fps_point.probability,
				// The false positive rate is the number of false_positives divided by the total number of negatives
				false_positive_rate: tps_fps_point.false_positives as f32 / count_negatives as f32,
			})
		}
		// compute the riemann sum using the trapezoidal rule
		roc_curve
			.iter()
			.tuple_windows()
			.map(|(left, right)| {
				let y_avg = (left.true_positive_rate + right.true_positive_rate) / 2.0;
				let dx = right.false_positive_rate - left.false_positive_rate;
				y_avg * dx
			})
			.sum()
	}
}

/// A point on the ROC curve, parameterized by thresholds.
#[derive(Debug, PartialEq)]
struct ROCCurvePoint {
	/// The classification threshold.
	threshold: f32,
	/// The true positive rate for all predictions with probability <= threshold.
	true_positive_rate: f32,
	/// The false positive rate for all predictions with probability <= threshold.
	false_positive_rate: f32,
}

#[derive(Debug)]
struct TpsFpsPoint {
	/// The prediction probability.
	probability: f32,
	/// The true positives for this threshold.
	true_positives: usize,
	/// The false positives for this threshold.
	false_positives: usize,
}

#[test]
fn test_roc_curve() {
	let labels = vec![2, 2, 1, 1];
	let probabilities = vec![0.9, 0.4, 0.4, 0.2];
	let actual = AUCROC::compute((probabilities.as_slice(), labels.as_slice()));
	let expected = 0.875;
	assert!(f32::abs(actual - expected) < f32::EPSILON)
}
