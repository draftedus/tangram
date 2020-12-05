use tangram_deps::html::{self, component, html};
use tangram_ui as ui;

#[component]
pub fn ClassSelectField(class: String, classes: Vec<String>) {
	html! {
		<ui::SelectField
			disabled={None}
			placeholder={None}
			required={None}
			id={"class-select-field".to_owned()}
			label={"Select Class".to_owned()}
			name={"class".to_owned()}
			options={classes.iter().map(|class_name| ui::SelectFieldOption {
				text: class_name.clone(),
				value: class_name.clone(),
			}).collect::<Vec<_>>()}
			value={class}
		/>
	}
}

pub fn boot_class_select_field() {
	ui::select_field_submit_on_change("class-select-field".to_owned());
}
