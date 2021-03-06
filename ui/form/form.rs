use html::{component, html};

#[component]
pub fn Form(
	action: Option<String>,
	autocomplete: Option<String>,
	enc_type: Option<String>,
	id: Option<String>,
	post: Option<bool>,
) {
	let method = post.and_then(|post| if post { Some("post".to_owned()) } else { None });
	html! {
		<form
			id={id}
			action={action}
			autocomplete={autocomplete}
			class="form"
			enctype={enc_type}
			method={method}
		>
			{children}
		</form>
	}
}
