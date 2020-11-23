use html::{component, html};

#[component]
pub fn Img(alt: String, src: String) {
	html! (
		<details class="image-details">
			<summary class="image-details-summary">
				<img alt={alt.clone()} class="image-img" src={src.clone()} />
			</summary>
			<div class="image-viewer">
				<img alt={alt} class="image-viewer-img" src={src} />
			</div>
		</details>
	)
}
