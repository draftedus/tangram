use super::Button;
use html::{component, html};

#[derive(Clone)]
pub struct TopbarItem {
	pub element: Option<html::Node>,
	pub href: String,
	pub title: String,
}

#[component]
pub fn Topbar(
	background_color: String,
	border: Option<String>,
	dropdown_background_color: String,
	foreground_color: String,
	items: Option<Vec<TopbarItem>>,
	logo: Option<html::Node>,
	logo_href: Option<String>,
	logo_img_url: Option<String>,
	title: Option<String>,
) {
	let mut wrapper_style = format!("background-color: {};", background_color);
	if let Some(border) = border.clone() {
		wrapper_style.push_str(&format!(" border-bottom: {};", border));
	}
	wrapper_style.push_str(&format!(" color: {};", foreground_color));
	html! {
		<div class="topbar-wrapper" style={wrapper_style}>
			<TopbarBrand
				logo_element={logo}
				logo_href={logo_href}
				logo_img_url={logo_img_url}
				text_color={foreground_color.clone()}
				title={title}
			/>
			{items.as_ref().map(|items| html! {
				<TopbarItemsWrapper>
					{items.iter().map(|item| {
						if let Some(element) = item.element.clone() {
							element
						} else {
							html! {
								<a class="topbar-link" href={item.href.clone()} key={item.title.clone()}>
									{item.title.clone()}
								</a>
							}
						}
					}).collect::<Vec<_>>()}
				</TopbarItemsWrapper>
			})}
			<details class="topbar-details">
				<summary class="topbar-details-summary">
					<TopbarHamburger text_color={foreground_color} />
				</summary>
				<TopbarDropdown
					background_color={dropdown_background_color}
					border={border}
					cta={None}
					items={items}
				/>
			</details>
		</div>
	}
}

#[component]
fn TopbarBrand(
	logo_element: Option<html::Node>,
	logo_href: Option<String>,
	logo_img_url: Option<String>,
	text_color: String,
	title: Option<String>,
) {
	let title_style = format!("color: {};", text_color);
	html! {
		<a class="topbar-link" href={logo_href.unwrap_or_else(|| "/".to_owned())}>
			<div class="topbar-brand-wrapper">
				{if let Some(logo_img_url) = logo_img_url { html!(
					<img class="topbar-brand-img" srcset={format!("{} 3x", logo_img_url)} />
				) } else { html!(
					<div class="topbar-brand-svg">{logo_element}</div>
				)}}
				{title.map(|title| html! {
					<div class="topbar-brand-title" style={title_style}>
						{title}
					</div>
				})}
			</div>
		</a>
	}
}

#[component]
fn TopbarItemsWrapper() {
	html! { <nav class="topbar-items-wrapper">{children}</nav> }
}

#[component]
fn TopbarHamburger(text_color: String) {
	html! {
		<div class="topbar-hamburger">
			<svg
				class="topbar-hamburger-icon"
				height="15px"
				overflow="visible"
				viewBox="0 0 1 1"
				width="15px"
			>
				{[0.0, 0.5, 1.0].iter().map(|y| html!(
					<line
						key={y.to_string()}
						stroke={text_color.clone()}
						stroke-linecap="round"
						stroke-width="0.2"
						x1="0"
						x2="1"
						y1={y.to_string()}
						y2={y.to_string()}
					/>
				)).collect::<Vec<_>>()}
			</svg>
			<svg
				class="topbar-x-icon"
				height="15px"
				overflow="visible"
				viewBox="0 0 1 1"
				width="15px"
			>
				<line
					stroke={text_color.clone()}
					stroke-linecap="round"
					stroke-width="0.2"
					x1="0"
					x2="1"
					y1="0"
					y2="1"
				/>
				<line
					stroke={text_color}
					stroke-linecap="round"
					stroke-width="0.2"
					x1="1"
					x2="0"
					y1="0"
					y2="1"
				/>
			</svg>
		</div>
	}
}

#[component]
fn TopbarDropdown(
	background_color: String,
	border: Option<String>,
	cta: Option<TopbarItem>,
	items: Option<Vec<TopbarItem>>,
) {
	let mut wrapper_style = format!("background-color: {};", background_color);
	if let Some(border) = border {
		wrapper_style.push_str(&format!(" border-bottom: {};", border));
	}
	html! {
		<div class="topbar-dropdown-wrapper" style={wrapper_style}>
			{items.map(|items| items.into_iter().map(|item| html! {
				<a class="topbar-dropdown-link" href={item.href} key={item.title.clone()}>
					<div class="topbar-dropdown-item" key={item.title.clone()}>
						{item.title}
					</div>
				</a>
			}).collect::<Vec<_>>())}
			{cta.map(|cta| {
				html! {
					<div class="topbar-dropdown-item">
						<Button
							disabled={None}
							download={None}
							id={None}
							href={Some(cta.href)}
						>
							{cta.title}
						</Button>
					</div>
				}
			})}
		</div>
	}
}
