/** Tokenizer that splits text into tokens. All non-alphanumeric characters are considered token boundaries. The text is lowercased before splitting.

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
