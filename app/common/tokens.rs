use html::{component, html};
use tangram_ui as ui;

pub const training_color: &str = ui::colors::BLUE;
pub const production_color: &str = ui::colors::GREEN;
pub const baseline_color: &str = ui::colors::GRAY;
pub const selected_threshold_color: &str = ui::colors::BLUE;

#[component]
pub fn UnknownColumnToken() {
	html! {
		<ui::Token color={Some("var(--gray)".to_owned())}>
			{"Unknown"}
		</ui::Token>
	}
}

#[component]
pub fn NumberColumnToken() {
	html! {
		<ui::Token color={Some("var(--teal)".to_owned())}>
			{"Number"}
		</ui::Token>
	}
}

#[component]
pub fn EnumColumnToken() {
	html! {
		<ui::Token color={Some("var(--purple)".to_owned())}>
			{"Enum"}
		</ui::Token>
	}
}

#[component]
pub fn TextColumnToken() {
	html! {
		<ui::Token color={Some("var(--orange)".to_owned())}>
			{"Text"}
		</ui::Token>
	}
}
