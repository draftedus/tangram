use crate::date_window::DateWindow;
use tangram_deps::html::{self, component, html};
use tangram_ui as ui;

#[component]
pub fn DateWindowSelectField(date_window: DateWindow) {
	let date_window_select_field_options = vec![
		ui::SelectFieldOption {
			text: "Today".to_owned(),
			value: "today".to_owned(),
		},
		ui::SelectFieldOption {
			text: "This Month".to_owned(),
			value: "this_month".to_owned(),
		},
		ui::SelectFieldOption {
			text: "This Year".to_owned(),
			value: "this_year".to_owned(),
		},
	];
	html! {
		<ui::SelectField
			disabled={None}
			placeholder={None}
			required={None}
			id={"date-window-select-field".to_owned()}
			label={"Date Window".to_owned()}
			name={"date_window".to_owned()}
			options={date_window_select_field_options}
			value={date_window.to_string()}
		/>
	}
}

pub fn boot_date_window_select_field() {
	ui::select_field_submit_on_change("date-window-select-field".to_owned());
}
