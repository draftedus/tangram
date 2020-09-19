use super::Metric;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct Mode(Option<(usize, usize)>);

impl<'a> Metric<'a> for Mode {
	type Input = &'a [usize];
	type Output = Option<usize>;
	fn compute(input: Self::Input) -> Self::Output {
		let mut histogram = BTreeMap::new();
		for value in input.iter() {
			*histogram.entry(value).or_insert(0) += 1;
		}
		histogram
			.into_iter()
			.max_by(|a, b| a.1.cmp(&b.1))
			.map(|label| *label.0)
	}
}
