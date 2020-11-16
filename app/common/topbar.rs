use super::logo::{Logo, LogoScheme};
use html::{component, html};
use tangram_ui as ui;

#[derive(Clone)]
pub struct TopbarAvatar {
	pub avatar_url: Option<String>,
}

#[component]
pub fn Topbar(topbar_avatar: Option<TopbarAvatar>) {
	let mut items = vec![];
	if let Some(topbar_avatar) = topbar_avatar {
		items.push(ui::TopbarItem {
			element: Some(html! {
				<ui::Link title={None} class_name={None} href={Some("/user".into())}>
					<ui::Avatar src={topbar_avatar.avatar_url} />
				</ui::Link>
			}),
			href: "/user".into(),
			title: "Settings".into(),
		})
	}
	html! {
		<ui::Topbar
			border={None}
			background_color={"var(--header-color)".into()}
			dropdown_background_color={"var(--surface-color)".into()}
			foreground_color={"var(--text-color)".into()}
			items={Some(items)}
			logo={Some(
				html! {
					<Logo class={None} color={None} color_scheme={LogoScheme::Multi} />
				}
			)}
			logo_href={Some("/".into())}
			logo_img_url={None}
			title={Some("tangram".into())}
			/>
	}
}
