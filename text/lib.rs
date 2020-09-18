/*!
The `tangram_text` crate provides text processing utilities used by [`tangram_core`](docs.rs/crates/tangram_core).
*/
use num_traits::ToPrimitive;

/// Computes the inverse document frequency given the total number of examples and the number of examples that contain a given term.
pub fn compute_idf(examples_count: u64, n_examples: u64) -> f32 {
	// This is the "inverse document frequency smooth" form of the idf, see https://en.wikipedia.org/wiki/Tf%E2%80%93idf. We add 1 to `n_examples_that_contain_token` to avoid division by 0.
	(n_examples.to_f32().unwrap() / (1.0 + examples_count.to_f32().unwrap())).ln() + 1.0
}

/** The `AlphanumericTokenizer` splits text into tokens. All non-alphanumeric characters are considered token boundaries. The text is lowercased before splitting. All tokens must be >1 character long.

# Example

| text        | tokens          |
|-------------|-----------------|
| Don't       | ["don"]         |
| $50         | ["50"]          |
| 50(hello)   | ["50", "hello"] |
| 50_hello    | ["50", "hello"] |
| C.E.O.      | []              |
*/
#[derive(Clone, Debug)]
pub struct AlphanumericTokenizer;

impl AlphanumericTokenizer {
	pub fn tokenize<'a>(&self, value: &'a str) -> Vec<String> {
		let mut tokens = Vec::new();
		let mut token = String::new();
		for c in value.to_lowercase().chars() {
			if !c.is_alphanumeric() {
				if token.len() > 1 {
					tokens.push(token.clone());
				}
				token.clear()
			} else {
				token.push(c)
			}
		}
		if token.len() > 1 {
			tokens.push(token);
		}
		tokens
	}
}

#[test]
fn test_basic_alphanumeric_tokenizer() {
	let tokenizer = AlphanumericTokenizer {};
	let tokens = tokenizer.tokenize("Don't");
	assert_eq!(tokens, vec!["don"]);
	let tokens = tokenizer.tokenize("CEO/Co-founder");
	assert_eq!(tokens, vec!["ceo", "co", "founder"]);
	let tokens = tokenizer.tokenize("CEO(Co-founder)");
	assert_eq!(tokens, vec!["ceo", "co", "founder"]);
	let tokens = tokenizer.tokenize("$50");
	assert_eq!(tokens, vec!["50"]);
	let tokens = tokenizer.tokenize("50_hello");
	assert_eq!(tokens, vec!["50", "hello"]);
	let tokens = tokenizer.tokenize("50(hello)");
	assert_eq!(tokens, vec!["50", "hello"]);
	let tokens = tokenizer.tokenize("C.E.O");
	assert!(tokens.is_empty());
	let tokens = tokenizer.tokenize("m/f");
	assert!(tokens.is_empty());
}

/// Computes bigrams by concatenating adjacent pairs of tokens with a space " ".
pub fn bigrams(tokens: &[String]) -> Vec<String> {
	let mut bigrams = Vec::new();
	let mut iter = tokens.iter().peekable();
	while let Some(a) = iter.next() {
		if let Some(b) = iter.peek() {
			bigrams.push(a.to_owned() + " " + b);
		}
	}
	bigrams
}

#[test]
fn test_ngrams() {
	let tokens = vec![];
	let bigram_tokens = bigrams(&tokens);
	assert!(bigram_tokens.is_empty());
	let tokens = vec!["a".into()];
	let bigram_tokens = bigrams(&tokens);
	assert!(bigram_tokens.is_empty());
	let tokens = vec!["a".into(), "b".into()];
	let bigram_tokens = bigrams(&tokens);
	assert_eq!(bigram_tokens, vec!["a b"]);
	let tokens = vec!["a".into(), "b".into(), "c".into()];
	let bigram_tokens = bigrams(&tokens);
	assert_eq!(bigram_tokens, vec!["a b", "b c"]);
}
