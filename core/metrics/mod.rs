mod accuracy;
mod auc_roc;
mod binary_classification;
mod binary_cross_entropy;
mod classification;
mod cross_entropy;
mod mean;
mod mean_squared_error;
mod mean_variance;
mod regression;

pub use accuracy::*;
pub use auc_roc::*;
pub use binary_classification::*;
pub use binary_cross_entropy::*;
pub use classification::*;
pub use cross_entropy::*;
pub use mean::*;
pub use mean_squared_error::*;
pub use mean_variance::*;
pub use regression::*;

/**
The `Metric` trait defines a common interface to compute metrics such as accuracy, precision, and recall, so that generic code can be written that computes arbitrary metrics.

After being initialized, a type `T` implementing the `Metric` trait can have `update()` called on it with values of the associated type `Input`. Multiple values of the type can be merged together by calling `merge()`. When finished aggregating, you can call `finalize()` on the metric to produce the associated type `Output`.

Here is a basic example implementation of a `Min` metric, which takes `f32`s and produces an `f32` that is the minimum of all the inputs.

struct Min(f32);

```rust
impl Metric for Min {
  type Input = f32;
  type Output = f32;
  fn update(&mut self, input: Self::Input) { self.0 = self.0.min(input) };
  fn merge(&mut self, other: Self) { self.0 = self.0.min(other.0) }
  fn finalize(self) -> Self::Output { self.0 }
}
```
*/
pub trait Metric<'a> {
	/// T
	type Input;
	type Output;
	///
	fn update(&mut self, input: Self::Input);
	/// Merge multiple values of this type.
	fn merge(&mut self, other: Self);
	/// When you are done aggregating `Input`s, call `finalize()` to produce an `Output`.
	fn finalize(self) -> Self::Output;
}
