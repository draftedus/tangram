use html::{component, html, raw};

#[derive(Clone)]
pub struct PageInfo {
	pub client_wasm_js_src: Option<String>,
}

#[component]
pub fn Document(page_info: PageInfo) {
	html! {
		<>
			{raw!("<!doctype html>")}
			<html lang="en">
				<head>
					<meta charset="utf-8" />
					<meta content="width=device-width, initial-scale=1" name="viewport" />
					<link href="/favicon.png" rel="icon" type="image/png" />
					<title>{"Tangram"}</title>
					<link href="/styles.css" rel="stylesheet" />
					<meta
						content="All-In-One Machine Learning Toolkit Designed for Programmers"
						name="description"
					/>
				</head>
				<body>
					{children}
					<script>
						{"document.cookie = `tangram-timezone=${Intl.DateTimeFormat().resolvedOptions().timeZone};max-age=31536000;samesite=lax`"}
					</script>
					{page_info.client_wasm_js_src.map(|client_wasm_js_src| html! {
						<script type="module">
							{raw!(format!(r#"import init from "{}"; init()"#, client_wasm_js_src))}
						</script>
					})}
				</body>
			</html>
		</>
	}
}
