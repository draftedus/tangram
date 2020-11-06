#[derive(Clone, Debug, Eq)]
pub struct TokenEntry(pub Token, pub usize);
impl std::cmp::Ord for TokenEntry {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.1.cmp(&other.1)
	}
}

impl std::cmp::PartialOrd for TokenEntry {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.1.partial_cmp(&other.1)
	}
}

impl std::cmp::PartialEq for TokenEntry {
	fn eq(&self, other: &Self) -> bool {
		self.1.eq(&other.1)
	}
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
	Unigram(String),
	Bigram(String, String),
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Token::Unigram(token) => write!(f, "{}", token),
			Token::Bigram(token_a, token_b) => write!(f, "{} {}", token_a, token_b),
		}
	}
}

/// This struct contains stats for individual tokens
#[derive(Debug)]
pub struct TokenStats {
	pub token: Token,
	/// This is the total number of occurrences of this token.
	pub count: usize,
	/// This is the total number of examples that contain this token.
	pub examples_count: usize,
	/// This is the inverse document frequency. [Learn more](https://en.wikipedia.org/wiki/Tf%E2%80%93idf).
	pub idf: f32,
}
