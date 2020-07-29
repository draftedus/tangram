use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse_macro_input;

enum Node {
	String(syn::LitStr),
	Block(syn::Block),
	Fragment(Fragment),
	Element(Element),
}

struct Fragment {
	pub children: Vec<Node>,
}

struct Element {
	pub name: syn::Path,
	pub attributes: Vec<Attribute>,
	pub children: Vec<Node>,
	pub self_closing: bool,
}

enum Attribute {
	Shorthand(AttributeKey),
	Longhand(AttributeKey, AttributeValue),
}

type AttributeKey = syn::punctuated::Punctuated<syn::Ident, syn::Token![-]>;

enum AttributeValue {
	String(syn::LitStr),
	Block(syn::Block),
}

impl syn::parse::Parse for Node {
	fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		if let Ok(result) = input.parse::<syn::LitStr>().map(Self::String) {
			Ok(result)
		} else if let Ok(result) = input.parse::<syn::Block>().map(Self::Block) {
			Ok(result)
		} else if input.peek(syn::Token![<]) && input.peek2(syn::Token![>]) {
			input.parse::<Fragment>().map(Self::Fragment)
		} else if input.peek(syn::Token![<]) {
			input.parse::<Element>().map(Self::Element)
		} else {
			Err(syn::Error::new(input.span(), "failed to parse node"))
		}
	}
}

impl syn::parse::Parse for Fragment {
	fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		input.parse::<syn::Token![<]>()?;
		input.parse::<syn::Token![>]>()?;
		let mut children = Vec::new();
		while !(input.peek(syn::Token![<]) && input.peek2(syn::Token![/])) {
			let child = input.parse::<Node>()?;
			children.push(child);
		}
		input.parse::<syn::Token![<]>()?;
		input.parse::<syn::Token![/]>()?;
		input.parse::<syn::Token![>]>()?;
		Ok(Self { children })
	}
}

impl syn::parse::Parse for Element {
	fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		input.parse::<syn::Token![<]>()?;
		let name = input.parse::<syn::Path>()?;
		let mut attributes = Vec::new();
		while !(input.peek(syn::Token![>]) || input.peek(syn::Token![/])) {
			let key = AttributeKey::parse_separated_nonempty_with(input, syn::Ident::parse_any)?;
			if !input.peek(syn::Token![=]) {
				attributes.push(Attribute::Shorthand(key));
			} else {
				input.parse::<syn::Token![=]>()?;
				let value = input
					.parse::<syn::LitStr>()
					.map(AttributeValue::String)
					.or_else(|_| input.parse::<syn::Block>().map(AttributeValue::Block))?;
				attributes.push(Attribute::Longhand(key, value));
			}
		}
		let self_closing = input.peek(syn::Token![/]);
		if self_closing {
			input.parse::<syn::Token![/]>()?;
		}
		input.parse::<syn::Token![>]>()?;
		let mut children = Vec::new();
		if !self_closing {
			while !(input.peek(syn::Token![<]) && input.peek2(syn::Token![/])) {
				let child = input.parse::<Node>()?;
				children.push(child);
			}
			input.parse::<syn::Token![<]>()?;
			input.parse::<syn::Token![/]>()?;
			let close_name = input.parse::<syn::Path>()?;
			if close_name != name {
				panic!();
			}
			input.parse::<syn::Token![>]>()?;
		}
		Ok(Self {
			name,
			attributes,
			children,
			self_closing,
		})
	}
}

impl quote::ToTokens for Node {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::String(string) => string.to_tokens(tokens),
			Self::Block(block) => block.to_tokens(tokens),
			Self::Fragment(fragment) => fragment.to_tokens(tokens),
			Self::Element(element) => element.to_tokens(tokens),
		}
	}
}

impl quote::ToTokens for Fragment {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let children = self.children.iter();
		let children = quote! { vec![#(#children.into()),*] };
		let code = quote! {
			::html::Node::Fragment(::html::FragmentNode {
				children: #children,
			})
		};
		code.to_tokens(tokens);
	}
}

impl quote::ToTokens for Element {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let name = self.name.get_ident().filter(|ident| {
			ident
				.to_string()
				.chars()
				.next()
				.unwrap()
				.is_ascii_lowercase()
		});
		if let Some(name) = name {
			let name = name.to_string();
			let attributes = self.attributes.iter().map(|attribute| {
				let (key, value) = match attribute {
					Attribute::Shorthand(key) => (key, quote! { #key.into() }),
					Attribute::Longhand(key, value) => match value {
						AttributeValue::String(string) => (key, quote! { #string.into() }),
						AttributeValue::Block(block) => (key, quote! { #block.into() }),
					},
				};
				let key = key
					.iter()
					.map(|key| key.to_string())
					.collect::<Vec<_>>()
					.join("");
				quote! { (#key, #value) }
			});
			let attributes = quote! { vec![#(#attributes),*] };
			let children = self.children.iter();
			let children = quote! { vec![#(#children.into()),*] };
			let self_closing = self.self_closing;
			let code = quote! {
				::html::Node::Host(::html::HostNode {
					name: #name,
					attributes: #attributes.into_iter().collect(),
					children: #children,
					self_closing: #self_closing,
				})
			};
			code.to_tokens(tokens);
		} else {
			let name = &self.name;
			let fields = self.attributes.iter().map(|attribute| match attribute {
				Attribute::Shorthand(key) => quote! { #key: #key },
				Attribute::Longhand(key, value) => match value {
					AttributeValue::String(string) => quote! { #key: #string },
					AttributeValue::Block(block) => {
						quote! { #key: #block }
					}
				},
			});
			let children = self.children.iter();
			let children = quote! { vec![#(#children.into()),*] };
			let code = quote! {
				::html::Node::Component(::html::ComponentNode::Unrendered {
					component: Some(Box::new(#name { #(#fields),* })),
					children: Some(#children),
				})
			};
			code.to_tokens(tokens);
		}
	}
}

fn component_transform(ast: syn::ItemFn) -> TokenStream {
	let visibility = ast.vis;
	let struct_name = ast.sig.ident;
	let (impl_generics, ty_generics, where_clause) = ast.sig.generics.split_for_impl();
	let inputs = ast.sig.inputs.iter().collect::<Vec<_>>();
	let block = ast.block;
	let input_patterns: Vec<_> = inputs
		.iter()
		.filter_map(|argument| match argument {
			syn::FnArg::Typed(typed) => {
				let pattern = &typed.pat;
				Some(quote!(#pattern))
			}
			_ => None,
		})
		.collect();
	let ast = quote! {
		#visibility struct #struct_name#impl_generics { #(#visibility #inputs),* }
		impl#impl_generics ::html::Component for #struct_name#ty_generics #where_clause {
			fn render(self: Box<Self>, children: Vec<html::Node>) -> html::Node {
				let #struct_name { #(#input_patterns),* } = *self;
				#block
			}
		}
	};
	ast.into()
}

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
	let ast = parse_macro_input!(input as Node);
	let ast = quote! { #ast };
	ast.into()
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, input: TokenStream) -> TokenStream {
	component_transform(parse_macro_input!(input as syn::ItemFn))
}
