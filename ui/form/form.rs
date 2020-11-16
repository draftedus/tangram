use html::{component, html};

#[component]
pub fn Form(
	action: Option<String>,
	auto_complete: Option<String>,
	enc_type: Option<String>,
	id: Option<String>,
	post: Option<bool>,
) {
	html! {
		<form
			id={id}
			action={action}
			auto_complete={auto_complete}
			class="form"
			enctype={enc_type}
			method={
				post.and_then(|post| if post {
					Some("post".to_owned())
				} else {
					None
				})
			}
		>
			{children}
		</form>
	}
}
