/*!
This crate defines the [`Metric`](trait.Metric.html) and [`StreamingMetric`](trait.StreamingMetric.html) traits and and a number of concrete types that implement them such as [`MeanSquaredError`](struct.MeanSquaredError.html) and [`Accuracy`](struct.Accuracy.html).
*/

#![allow(clippy::tabs_in_doc_comments)]

mod accuracy;
mod auc_roc;
mod binary_classification;
mod binary_cross_entropy;
mod cross_entropy;
mod mean;
mod mean_squared_error;
mod mode;
mod multiclass_classification;
mod regression;

pub use self::accuracy::Accuracy;
pub use self::auc_roc::*;
pub use self::binary_classification::{
	BinaryClassificationMetrics, BinaryClassificationMetricsInput,
	BinaryClassificationMetricsOutput, BinaryClassificationMetricsOutputForThreshold,
};
pub use self::binary_cross_entropy::{BinaryCrossEntropy, BinaryCrossEntropyInput};
pub use self::cross_entropy::{CrossEntropy, CrossEntropyInput, CrossEntropyOutput};
pub use self::mean::Mean;
pub use self::mean_squared_error::MeanSquaredError;
pub use self::mode::Mode;
pub use self::multiclass_classification::{
	ClassMetrics, MulticlassClassificationMetrics, MulticlassClassificationMetricsInput,
	MulticlassClassificationMetricsOutput,
};
pub use self::regression::{
	m2_to_variance, merge_mean_m2, RegressionMetrics, RegressionMetricsInput,
	RegressionMetricsOutput,
};

/**
The `Metric` trait defines a common interface to metrics that can be computed when the entire input is available at once.

The seemingly unused generic lifetime `'a` exists here to allow `Input`s and `Output`s to borrow from their enclosing scope. When Rust stabilizes Generic Associated Types (GATs), the generic lifetime will move to the associated types.
*/
pub trait Metric<'a> {
	type Input;
	type Output;
	fn compute(input: Self::Input) -> Self::Output;
}

/**
The `StreamingMetric` trait defines a common interface to metrics that can be computed in a streaming manner, where the input is available in chunks, such as mean squared error and accuracy.

After being initialized, a value of type `T` implementing the `StreamingMetric` trait can have `update()` called on it with values of the associated type `Input`. Multiple values of `T` can be merged together by calling `merge()`. This is useful when computing a metric across multiple threads. When finished aggregating, you can call `finalize()` on the metric to produce the associated type `Output`.

# Examples

Here is a basic example implementation of a `Min` metric, which takes `f32`s as input and produces an `f32` as output that is the minimum of all the inputs.

```
use tangram_metrics::StreamingMetric;

struct Min(f32);

impl StreamingMetric<'_> for Min {
	type Input = f32;
	type Output = f32;
	fn update(&mut self, input: Self::Input) {
		self.0 = self.0.min(input)
	}
	fn merge(&mut self, other: Self) { self.0 = self.0.min(other.0) }
	fn finalize(self) -> Self::Output { self.0 }
}
```

The seemingly unused generic lifetime `'a` exists here to allow `Input`s and `Output`s to borrow from their enclosing scope. When Rust stabilizes Generic Associated Types (GATs), the generic lifetime will move to the associated types.

*/
pub trait StreamingMetric<'a> {
	/// `Input` is the type to aggregate in calls to `update()`.
	type Input;
	/// `Output` is the return type of `finalize()`.
	type Output;
	/// Update this streaming metric with the `Input` `input`.
	fn update(&mut self, input: Self::Input);
	/// Merge multiple independently computed streaming metrics.
	fn merge(&mut self, other: Self);
	/// When you are done aggregating `Input`s, call `finalize()` to produce an `Output`.
	fn finalize(self) -> Self::Output;
}
