use super::Metric;

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
		// Compute the counts of true positives and false positives at each classification threshold. Unlike the roc curve, each point contains just the count of true positives and false positives at this threshold instead of the cumulative sum of true positives and false positives up to this threshold.
		let mut tps_fps: Vec<TpsFpsPoint> = Vec::new();
		for (probability, label) in probabilities_labels.iter() {
			let label = label.checked_sub(1).unwrap();
			let tp = label;
			match tps_fps.last() {
				Some(last_point)
					if probability.partial_cmp(&last_point.threshold).unwrap()
						== std::cmp::Ordering::Equal =>
				{
					let last = tps_fps.last_mut().unwrap();
					last.true_positives += tp;
					last.false_positives += 1 - tp;
				}
				_ => {
					tps_fps.push(TpsFpsPoint {
						threshold: *probability,
						true_positives: tp,
						false_positives: 1 - tp,
					});
				}
			}
		}
		for i in 1..tps_fps.len() {
			tps_fps[i].true_positives += tps_fps[i - 1].true_positives;
			tps_fps[i].false_positives += tps_fps[i - 1].false_positives;
		}
		let count_positives = labels
			.iter()
			.map(|l| l.checked_sub(1).unwrap())
			.sum::<usize>();
		let count_negatives = labels.len() - count_positives;
		// add a point at (0,0) on the roc curve with a dummy threshold of 1.0
		let mut roc_curve = vec![ROCCurvePoint {
			threshold: 1.0,
			true_positive_rate: 0.0,
			false_positive_rate: 0.0,
		}];
		for tps_fps_point in tps_fps.iter() {
			roc_curve.push(ROCCurvePoint {
				true_positive_rate: tps_fps_point.true_positives as f32 / count_positives as f32,
				threshold: tps_fps_point.threshold,
				false_positive_rate: tps_fps_point.false_positives as f32 / count_negatives as f32,
			})
		}
		(0..roc_curve.len() - 1)
			.map(|i| {
				let left = &roc_curve[i];
				let right = &roc_curve[i + 1];
				let y_left = left.true_positive_rate;
				let y_right = right.true_positive_rate;
				let y_average = (y_left + y_right) / 2.0;
				let dx = right.false_positive_rate - left.false_positive_rate;
				y_average * dx
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
	/// The classification threshold.
	threshold: f32,
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
