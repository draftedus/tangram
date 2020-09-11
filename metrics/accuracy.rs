use super::Metric;
use itertools::izip;
use ndarray::prelude::*;
use num_traits::ToPrimitive;
use std::num::NonZeroU64;

#[derive(Debug, Clone, Default)]
pub struct Accuracy(Option<(NonZeroU64, u64)>);

pub struct AccuracyInput<'a> {
	probabilities: ArrayView1<'a, f32>,
	label: usize,
}

impl<'a> Metric<'a> for Accuracy {
	type Input = AccuracyInput<'a>;
	type Output = Option<f32>;

	fn update(&mut self, value: Self::Input) {
		let one = NonZeroU64::new(1u64).unwrap();
		let correct = if is_correct(value.probabilities, value.label) {
			1
		} else {
			0
		};
		self.0 = match self.0 {
			None => Some((one, correct)),
			Some(accuracy) => Some((
				NonZeroU64::new(accuracy.0.get() + 1).unwrap(),
				accuracy.1 + correct,
			)),
		}
	}

	fn merge(&mut self, other: Self) {
		self.0 = match (self.0, other.0) {
			(None, None) => None,
			(None, Some((n, correct))) => Some((n, correct)),
			(Some((n, correct)), None) => Some((n, correct)),
			(Some((n_a, correct_a)), Some((n_b, correct_b))) => Some((
				NonZeroU64::new(n_a.get() + n_b.get()).unwrap(),
				correct_a + correct_b,
			)),
		};
	}

	fn finalize(self) -> Option<f32> {
		self.0
			.map(|(n, correct)| correct.to_f32().unwrap() / n.get().to_f32().unwrap())
	}
}

/// compute the accuracy given probabilities and labels
/// where probabilities have shape (n_examples, n_classes)
/// and labels have shape (n_examples)
pub fn accuracy(probabilities: ArrayView2<f32>, labels: ArrayView1<usize>) -> f32 {
	let n_examples = probabilities.nrows();
	let n_correct: usize = izip!(probabilities.axis_iter(Axis(0)), labels.as_slice().unwrap())
		.map(|(probabilities, label)| {
			if is_correct(probabilities, *label) {
				1
			} else {
				0
			}
		})
		.sum();
	n_correct.to_f32().unwrap() / n_examples.to_f32().unwrap()
}

fn is_correct(probabilities: ArrayView1<f32>, label: usize) -> bool {
	// labels are 1-indexed, convert to 0-indexed
	let label = label.checked_sub(1).unwrap();
	// get the index of the max probability
	let prediction = probabilities
		.iter()
		.enumerate()
		.max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
		.unwrap()
		.0;
	prediction == label
}
