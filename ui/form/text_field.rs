use super::FieldLabel;
use html::{component, html};

#[component]
pub fn TextField(
	autocomplete: Option<String>,
	disabled: Option<bool>,
	label: Option<String>,
	name: Option<String>,
	placeholder: Option<String>,
	readonly: Option<bool>,
	required: Option<bool>,
	value: Option<String>,
) {
	html! {
		<FieldLabel html_for={None}>
			{label}
			<input
				autocomplete={autocomplete}
				class="form-text-field"
				disabled={disabled}
				name={name}
				placeholder={placeholder}
				readonly={readonly}
				required={required}
				spellcheck={false}
				value={value}
			/>
		</FieldLabel>
	}
}
