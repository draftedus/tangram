use super::FieldLabel;
use html::{component, html};

#[component]
pub fn CheckboxField(
	label: Option<String>,
	name: Option<String>,
	placeholder: Option<String>,
	readonly: Option<bool>,
	value: Option<String>,
) {
	html! {
		<FieldLabel html_for={None}>
			{label}
			<input
				class="form-checkbox-field"
				name={name}
				placeholder={placeholder}
				readonly={readonly}
				type="checkbox"
				value={value}
			/>
		</FieldLabel>
	}
}
