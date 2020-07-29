use html::{component, html};

#[component]
pub fn AppLayout() {
	html!(
		<>
			{html::RawTextNode("<!doctype html>".into())}
			<html>
				<head>
					<meta charset="utf-8" />
					<link rel="stylesheet" href="/tangram.css" />
					<style>{include_str!("app_layout.css")}</style>
					<style>{include_str!("topbar_layout.css")}</style>
				</head>
				<body>
					<TopbarLayout>
						<div class="app-layout-grid">{props.children}</div>
					</TopbarLayout
				</body>
			</html>
		</>
	)
}
