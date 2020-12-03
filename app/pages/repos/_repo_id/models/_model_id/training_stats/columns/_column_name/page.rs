use super::enum_column::EnumColumn;
use super::number_column::NumberColumn;
use super::text_column::TextColumn;
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelLayoutInfo, ModelSideNavItem},
};
use tangram_deps::html::{self, html};

pub struct Props {
	pub inner: Inner,
	pub model_layout_info: ModelLayoutInfo,
}

pub enum Inner {
	Number(NumberProps),
	Enum(EnumProps),
	Text(TextProps),
}

#[derive(Clone)]
pub struct NumberProps {
	pub invalid_count: u64,
	pub max: f32,
	pub mean: f32,
	pub min: f32,
	pub name: String,
	pub p25: f32,
	pub p50: f32,
	pub p75: f32,
	pub std: f32,
	pub unique_count: u64,
}

#[derive(Clone)]
pub struct EnumProps {
	pub histogram: Option<Vec<(String, u64)>>,
	pub invalid_count: u64,
	pub name: String,
	pub unique_count: u64,
}

#[derive(Clone)]
pub struct TextProps {
	pub name: String,
	pub n_tokens: usize,
	pub tokens: Vec<TokenStats>,
}

#[derive(Clone)]
pub struct TokenStats {
	pub token: String,
	pub count: u64,
	pub examples_count: u64,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let inner = match props.inner {
		Inner::Number(inner) => html! {
			<NumberColumn props={inner} />
		},
		Inner::Enum(inner) => html! {
			<EnumColumn props={inner} />
		},
		Inner::Text(inner) => html! {
			<TextColumn props={inner} />
		},
	};
	let html = html! {
		<ModelLayout
			info={props.model_layout_info}
			page_info={page_info}
			selected_item={ModelSideNavItem::TrainingStats}
		>
			{inner}
		</ModelLayout>
	};
	html.render_to_string()
}
