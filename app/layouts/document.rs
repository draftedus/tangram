use html::{component, html};

#[component]
pub fn Document() {
	html! {
		<html lang="en">
			<head>
				<meta charset="utf-8" />
				<meta content="width=device-width, initial-scale=1" name="viewport" />
				<link href="/favicon.png" rel="icon" type="image/png" />
				<title>{"Tangram"}</title>
				<link href="/ui.css" rel="stylesheet" />
				<link href="/charts.css" rel="stylesheet" />
				<link href="/app.css" rel="stylesheet" />
				<link href="/www.css" rel="stylesheet" />
				<meta
					content="All-In-One Machine Learning Toolkit for Programmers"
					name="description"
				/>
			</head>
			<body>
				{children}
				<script>
					{"document.cookie = `tangram-timezone=${Intl.DateTimeFormat().resolvedOptions().timeZone};max-age=31536000`"}
				</script>
			</body>
		</html>
	}
}
