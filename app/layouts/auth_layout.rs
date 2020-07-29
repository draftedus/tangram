use html::{component, html};

pub mod colors {
	pub static BLUE: &str = "#0A84FF";
	pub static GRAY: &str = "#8E8E93";
	pub static GREEN: &str = "#30D158";
	pub static INDIGO: &str = "#5e5ce6";
	pub static ORANGE: &str = "#FF9F0A";
	pub static PINK: &str = "#FF375F";
	pub static PURPLE: &str = "#BF5AF2";
	pub static RED: &str = "#FF453A";
	pub static TEAL: &str = "#4DD0E1";
	pub static YELLOW: &str = "#FFD60A";
}

enum LogoScheme {
	Multi,
	Solid,
}

#[component]
pub fn Logo() {
	let trapezoid = colors::PINK;
	let square = colors::YELLOW;
	let medium_triangle = colors::TEAL;
	let small_triangle_1 = colors::PURPLE;
	let small_triangle_2 = colors::INDIGO;
	let large_triangle_1 = colors::BLUE;
	let large_triangle_2 = colors::GREEN;
	html! (
		<svg  height="100%" viewBox="0 0 200 200" width="100%">
			<desc>"tangram"</desc>
			<polygon
				fill={trapezoid}
				points="4 9.657 4 98.343 46 140.343 46 51.657"
			/>
			<polygon
				fill={square}
				points="100 105.657 55.657 150 100 194.343 144.343 150"
			/>
			<polygon
				fill={medium_triangle}
				points="4 109.657 4 196 90.343 196"
			/>
			<polygon
				fill={small_triangle_1}
				points="54 59.657 54 140.343 94.343 100"
			/>
			<polygon
				fill={small_triangle_2}
				points="150 155.657 109.657 196 190.343 196"
			/>
			<polygon
				fill={large_triangle_1}
				points="190.343 4 9.657 4 100 94.343"
			/>
			<polygon
				fill={large_triangle_2}
				points="196 9.657 105.657 100 196 190.343"
			/>
		</svg>
	)
}

#[component]
pub fn AuthLayout() {
	html!(
		<>
			{html::RawTextNode("<!doctype html>".into())}
			<html>
				<head>
					<meta charset="utf-8" />
					<link rel="stylesheet" href="/tangram.css" />
					<style>{include_str!("auth_layout.css")}</style>
				</head>
				<body>
					<div class="auth-layout-grid">
						<div class="auth-layout-logo">
							<Logo />
						</div>
						<div class="auth-layout-card">
							<div class="card" padding="2rem">{children}</div>
						</div>
					</div>
				</body>
			</html>
		</>
	)
}
