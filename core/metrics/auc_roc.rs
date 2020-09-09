// computes the auc_roc given labels and probabilities
pub fn auc_roc(probabilities: &[f32], labels: &[usize]) -> f32 {
	let roc_curve = compute_roc_curve(probabilities, labels);
	// compute the riemann sum of the auc_roc_curve
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

#[derive(Debug, std::cmp::PartialEq)]
pub struct ROCCurvePoint {
	pub threshold: f32,
	pub true_positive_rate: f32,
	pub false_positive_rate: f32,
}

pub fn compute_roc_curve(probabilities: &[f32], labels: &[usize]) -> Vec<ROCCurvePoint> {
	let mut tps_fps = compute_tps_fps_by_threshold(probabilities, labels);
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
		threshold: 1.1,
		true_positive_rate: 0.0,
		false_positive_rate: 0.0,
	}];
	tps_fps.iter().for_each(|tps_fps_point| {
		roc_curve.push(ROCCurvePoint {
			true_positive_rate: tps_fps_point.true_positives as f32 / count_positives as f32,
			threshold: tps_fps_point.threshold,
			false_positive_rate: tps_fps_point.false_positives as f32 / count_negatives as f32,
		})
	});
	roc_curve
}

#[derive(Debug)]
pub struct TpsFpsPoint {
	pub threshold: f32,
	pub true_positives: usize,
	pub false_positives: usize,
}

// collects the tps/fps in each threshold
// unlike roc curve, each point is not cumulative from the last
fn compute_tps_fps_by_threshold(probabilities: &[f32], labels: &[usize]) -> Vec<TpsFpsPoint> {
	let mut probabilities_labels: Vec<(f32, usize)> = probabilities
		.into_iter()
		.zip(labels.into_iter())
		.map(|(a, b)| (a.to_owned(), b.to_owned()))
		.collect();
	probabilities_labels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
	probabilities_labels.reverse();
	let mut tps_fps: Vec<TpsFpsPoint> = Vec::new();
	probabilities_labels
		.iter()
		.for_each(|(probability, label)| {
			// if probability is same as the last one, add to the previous bucket
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
		});

	tps_fps
}

#[test]
fn test_roc_curve() {
	let labels = vec![2, 2, 1, 1];
	let probabilities = vec![0.9, 0.4, 0.4, 0.2];
	let left = compute_roc_curve(probabilities.as_slice(), labels.as_slice());
	let right = vec![
		ROCCurvePoint {
			threshold: 1.0,
			true_positive_rate: 0.0,
			false_positive_rate: 0.0,
		},
		ROCCurvePoint {
			threshold: 0.9,
			true_positive_rate: 0.5,
			false_positive_rate: 0.0,
		},
		ROCCurvePoint {
			threshold: 0.4,
			true_positive_rate: 1.0,
			false_positive_rate: 0.5,
		},
		ROCCurvePoint {
			threshold: 0.2,
			true_positive_rate: 1.0,
			false_positive_rate: 1.0,
		},
	];
	left.iter()
		.zip(right.iter())
		.for_each(|(left, right)| assert_eq!(left, right));
	let auc = auc_roc(probabilities.as_slice(), labels.as_slice());
	assert_eq!(auc, 0.875)
}
