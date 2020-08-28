use num_traits::ToPrimitive;
use rand::random;
use tangram_core::metrics::RunningMetric;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberStats {
	pub n: u64,
	pub min: f32,
	pub max: f32,
	pub mean: f64,
	pub m2: f64,
	/// used to get an estimate for quantiles
	pub reservoir: Vec<f32>,
	pub reservoir_size: usize,
}

#[derive(Debug)]
pub struct NumberStatsOutput {
	pub n: u64,
	pub min: f32,
	pub max: f32,
	pub mean: f32,
	pub variance: f32,
	pub std: f32,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
}

impl NumberStats {
	pub fn new(value: f32) -> Self {
		Self {
			n: 1,
			min: value,
			max: value,
			mean: value.to_f64().unwrap(),
			m2: 0.0,
			reservoir: vec![value],
			reservoir_size: 100,
		}
	}
}

impl RunningMetric<'_, '_> for NumberStats {
	type Input = f32;
	type Output = NumberStatsOutput;

	fn update(&mut self, value: Self::Input) {
		let (new_mean, new_m2) = tangram_core::metrics::merge_mean_m2(
			self.n,
			self.mean,
			self.m2,
			1,
			value.to_f64().unwrap(),
			0.0,
		);
		self.n += 1;
		self.mean = new_mean;
		self.m2 = new_m2;
		self.min = f32::min(self.min, value);
		self.max = f32::max(self.max, value);
		if self.reservoir.len() < self.reservoir_size {
			self.reservoir.push(value)
		} else {
			let index = (random::<f32>() * self.n.to_f32().unwrap())
				.floor()
				.to_usize()
				.unwrap();
			if index < self.reservoir_size {
				self.reservoir[index] = value;
			}
		}
	}

	fn merge(&mut self, other: Self) {
		let (new_mean, new_m2) = tangram_core::metrics::merge_mean_m2(
			self.n, self.mean, self.m2, other.n, other.mean, other.m2,
		);
		self.n += other.n;
		self.mean = new_mean;
		self.m2 = new_m2;
		self.min = f32::min(self.min, other.min);
		self.max = f32::max(self.max, other.max);
		self.reservoir.extend(other.reservoir);
	}

	fn finalize(self) -> Self::Output {
		let reservoir_len = self.reservoir.len().to_f32().unwrap();
		let quantiles: Vec<f32> = vec![0.25, 0.50, 0.75];
		// find the index of each quantile given the total number of values in the dataset
		let quantile_indexes: Vec<usize> = quantiles
			.iter()
			.map(|q| ((reservoir_len - 1.0) * q).trunc().to_usize().unwrap())
			.collect();
		// the fractiononal part of the index
		// used to interpolate values if the index is not an integer value
		let quantile_fracts: Vec<f32> = quantiles
			.iter()
			.map(|q| ((reservoir_len - 1.0) * q).fract())
			.collect();
		let mut quantiles: Vec<f32> = vec![0.0; quantiles.len()];
		let mut samples = self.reservoir;
		samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
		quantiles
			.iter_mut()
			.zip(quantile_indexes.iter().zip(quantile_fracts))
			.for_each(|(quantile, (index, fract))| {
				let value = samples[*index];
				if fract > 0.0 {
					let next_value = samples[index + 1];
					// interpolate between two values
					*quantile = value * (1.0 - fract) + next_value * fract;
				} else {
					*quantile = value;
				}
			});
		Self::Output {
			n: self.n,
			p25: quantiles[0],
			p50: quantiles[1],
			p75: quantiles[2],
			mean: self.mean.to_f32().unwrap(),
			variance: tangram_core::metrics::m2_to_variance(self.m2, self.n),
			std: tangram_core::metrics::m2_to_variance(self.m2, self.n).sqrt(),
			min: self.min,
			max: self.max,
		}
	}
}
