use tangram_deps::html::{self, component, html};

#[component]
pub fn MetricsRow() {
	html! {
		<div class="metrics-row">
		  {children}
	  </div>
	}
}
