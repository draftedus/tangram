/*!
This module provides text processing utilities used by [`tangram_core`](docs.rs/crates/tangram_core).
*/

use std::borrow::Cow;

/**
An `AlphanumericTokenizer` splits text into tokens of adjacent alphanumeric characters. The tokens are lowercased. All tokens must be more than one character long.

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
pub struct AlphanumericTokenizer<'a> {
	text: &'a str,
	byte_index: usize,
}

impl<'a> AlphanumericTokenizer<'a> {
	pub fn new(text: &'a str) -> Self {
		Self {
			text,
			byte_index: 0,
		}
	}
}

impl<'a> Iterator for AlphanumericTokenizer<'a> {
	type Item = Cow<'a, str>;
	fn next(&mut self) -> Option<Self::Item> {
		// Find the next pair of two non-alphanumeric chars.
		loop {
			// Get the next char.
			let next_char = match self.text[self.byte_index..].chars().next() {
				Some(c) => c,
				None => return None,
			};
			// Get the next next char.
			let next_next_char = match self.text[self.byte_index + next_char.len_utf8()..]
				.chars()
				.next()
			{
				Some(c) => c,
				None => return None,
			};
			// If both the next and next next chars are alphanumeric, we can continue, we have found the start of a token. If not, pass over the next char and try again.
			if next_char.is_alphanumeric() && next_next_char.is_alphanumeric() {
				break;
			} else {
				self.byte_index += next_char.len_utf8();
			}
		}
		// This token will start at the current index.
		let start = self.byte_index;
		let mut contains_capital_letter = false;
		// Pass over as many adjacent alphanumeric characters as we can.
		while let Some(next_char) = self.text[self.byte_index..].chars().next() {
			if next_char.is_alphanumeric() {
				if next_char.is_uppercase() {
					contains_capital_letter = true;
				}
				self.byte_index += next_char.len_utf8();
				continue;
			} else {
				break;
			}
		}
		let end = self.byte_index;
		let token = &self.text[start..end];
		// Convert to lowercase only if the token contained any uppercase letters.
		let token = if contains_capital_letter {
			Cow::Owned(token.to_lowercase())
		} else {
			Cow::Borrowed(token)
		};
		Some(token)
	}
}

#[test]
fn test_basic_alphanumeric_tokenizer() {
	fn test(text: &str, tokens: &[&str]) {
		assert!(AlphanumericTokenizer::new(text).eq(tokens.iter().cloned()));
	}
	test("Don't", &["don"]);
	test("CEO/Co-founder", &["ceo", "co", "founder"]);
	test("CEO(Co-founder)", &["ceo", "co", "founder"]);
	test("$50", &["50"]);
	test("50_hello", &["50", "hello"]);
	test("50(hello)", &["50", "hello"]);
	test("C.E.O", &[]);
	test("m/f", &[]);
}
