mod accuracy;
mod binary_classification;
mod binary_cross_entropy;
mod classification;
mod cross_entropy;
mod mean;
mod mean_squared_error;
mod mean_variance;
mod regression;

pub use accuracy::*;
pub use binary_classification::*;
pub use binary_cross_entropy::*;
pub use classification::*;
pub use cross_entropy::*;
pub use mean::*;
pub use mean_squared_error::*;
pub use mean_variance::*;
pub use regression::*;

pub trait RunningMetric<'a, 'b> {
	type Input;
	type Output;
	fn update(&mut self, input: Self::Input);
	fn merge(&mut self, other: Self);
	fn finalize(self) -> Self::Output;
}
