mod bag_of_words;
mod compute;
mod identity;
mod normalized;
mod one_hot_encoded;

pub use self::bag_of_words::*;
pub use self::compute::*;
pub use self::identity::*;
pub use self::normalized::*;
pub use self::one_hot_encoded::*;

/// The `FeatureGroup` struct describes how to transform one or more columns from the input dataframe to one or more columns in the output features.
#[derive(Debug)]
pub enum FeatureGroup {
	Identity(self::identity::IdentityFeatureGroup),
	Normalized(self::normalized::NormalizedFeatureGroup),
	OneHotEncoded(self::one_hot_encoded::OneHotEncodedFeatureGroup),
	BagOfWords(self::bag_of_words::BagOfWordsFeatureGroup),
}

impl FeatureGroup {
	/// Return the number of features this feature group will produce.
	pub fn n_features(&self) -> usize {
		match self {
			FeatureGroup::Identity(_) => 1,
			FeatureGroup::Normalized(_) => 1,
			FeatureGroup::OneHotEncoded(s) => s.options.len() + 1,
			FeatureGroup::BagOfWords(s) => s.tokens.len(),
		}
	}
}
