use derive_more::From;
use std::borrow::Cow;
use std::fmt::Write;

pub use html_macro::{component, html};

#[derive(Clone, From)]
pub enum Node {
	RawText(RawTextNode),
	EscapedText(EscapedTextNode),
	Fragment(FragmentNode),
	Host(HostNode),
	Component(ComponentNode),
	Option(Option<Box<Node>>),
	Vec(Vec<Node>),
}

#[derive(Clone)]
pub struct RawTextNode(pub Cow<'static, str>);

#[derive(Clone)]
pub struct EscapedTextNode(pub Cow<'static, str>);

#[derive(Clone)]
pub struct FragmentNode {
	pub children: Vec<Node>,
}

#[derive(Clone)]
pub struct HostNode {
	pub name: &'static str,
	pub attributes: Vec<(AttributeKey, AttributeValue)>,
	pub children: Vec<Node>,
	pub self_closing: bool,
}

pub type AttributeKey = &'static str;

#[derive(Clone, From)]
pub enum AttributeValue {
	Bool(Option<bool>),
	String(Option<Cow<'static, str>>),
}

#[derive(Clone)]
pub enum ComponentNode {
	Unrendered {
		component: Option<Box<dyn Component>>,
		children: Option<Vec<Node>>,
	},
	Rendered(Box<Node>),
}

pub trait Component: ComponentClone {
	fn render(self: Box<Self>, children: Vec<Node>) -> Node;
}

pub trait ComponentClone {
	fn clone_box(&self) -> Box<dyn Component>;
}

impl<T> ComponentClone for T
where
	T: Component + Clone + 'static,
{
	fn clone_box(&self) -> Box<dyn Component> {
		Box::new(self.clone())
	}
}

impl Clone for Box<dyn Component> {
	fn clone(&self) -> Box<dyn Component> {
		self.clone_box()
	}
}

impl Node {
	pub fn render_to_string(mut self) -> String {
		self.render().to_string()
	}
	fn render(&mut self) -> &mut Node {
		match self {
			Node::Fragment(node) => {
				for child in node.children.iter_mut() {
					child.render();
				}
			}
			Node::Host(node) => {
				for child in node.children.iter_mut() {
					child.render();
				}
			}
			Node::Component(node) => {
				if let ComponentNode::Unrendered {
					component,
					children,
				} = node
				{
					let component = component.take().unwrap();
					let children = children.take().unwrap();
					let mut rendered = component.render(children);
					rendered.render();
					*node = ComponentNode::Rendered(Box::new(rendered));
				}
			}
			Node::Vec(node) => {
				for child in node.iter_mut() {
					child.render();
				}
			}
			Node::Option(node) => {
				if let Some(node) = node {
					node.render();
				}
			}
			_ => {}
		};
		self
	}
}

impl std::fmt::Display for Node {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Node::RawText(node) => {
				write!(f, "{}", node)?;
			}
			Node::EscapedText(node) => {
				write!(f, "{}", node)?;
			}
			Node::Fragment(node) => {
				for child in &node.children {
					write!(f, "{}", child)?;
				}
			}
			Node::Host(node) => {
				write!(f, "{}", node)?;
			}
			Node::Component(node) => {
				write!(f, "{}", node)?;
			}
			Node::Option(node) => {
				if let Some(node) = node {
					write!(f, "{}", node)?;
				}
			}
			Node::Vec(node) => {
				for node in node {
					write!(f, "{}", node)?;
				}
			}
		};
		Ok(())
	}
}

impl std::fmt::Display for FragmentNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for child in self.children.iter() {
			write!(f, "{}", child)?;
		}
		Ok(())
	}
}

impl std::fmt::Display for HostNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<{}", self.name)?;
		for (key, value) in self.attributes.iter() {
			match value {
				AttributeValue::Bool(value) => {
					if let Some(true) = value {
						write!(f, " {}", key)?;
					}
				}
				AttributeValue::String(value) => {
					if let Some(value) = value {
						write!(f, r#" {}="{}""#, key, value)?;
					}
				}
			}
		}
		if self.self_closing {
			write!(f, " /")?;
		}
		write!(f, ">")?;
		if !self.self_closing {
			for child in self.children.iter() {
				write!(f, "{}", child)?;
			}
			write!(f, "</{}>", self.name)?;
		}
		Ok(())
	}
}

impl std::fmt::Display for ComponentNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let rendered = match self {
			ComponentNode::Rendered(r) => r,
			_ => panic!("attempted to display component that has not yet been rendered"),
		};
		write!(f, "{}", rendered)?;
		Ok(())
	}
}

impl std::fmt::Display for RawTextNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl std::fmt::Display for EscapedTextNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for c in self.0.chars() {
			match c {
				'>' => write!(f, "&gt;")?,
				'<' => write!(f, "&lt;")?,
				'"' => write!(f, "&quot;")?,
				'&' => write!(f, "&amp;")?,
				'\'' => write!(f, "&apos;")?,
				c => f.write_char(c)?,
			};
		}
		Ok(())
	}
}

impl From<Option<String>> for AttributeValue {
	fn from(value: Option<String>) -> AttributeValue {
		AttributeValue::String(value.map(|value| value.into()))
	}
}

impl From<String> for AttributeValue {
	fn from(value: String) -> AttributeValue {
		AttributeValue::String(Some(value.into()))
	}
}

impl From<&'static str> for AttributeValue {
	fn from(value: &'static str) -> AttributeValue {
		AttributeValue::String(Some(value.into()))
	}
}

impl From<String> for Node {
	fn from(value: String) -> Node {
		Node::EscapedText(EscapedTextNode(value.into()))
	}
}

impl From<&'static str> for Node {
	fn from(value: &'static str) -> Node {
		Node::EscapedText(EscapedTextNode(value.into()))
	}
}

impl<T> From<Option<T>> for Node
where
	T: Into<Node>,
{
	fn from(value: Option<T>) -> Node {
		Node::Option(value.map(|value| Box::new(value.into())))
	}
}

#[macro_export]
macro_rules! raw {
	($t:expr) => {
		::html::RawTextNode($t.into())
	};
}

#[macro_export]
macro_rules! text {
	($t:expr) => {
		::html::EscapedTextNode($t.into())
	};
}
