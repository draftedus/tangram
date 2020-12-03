use html::{component, html};
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
