use super::logo::{Logo, LogoScheme};
use html::{component, html};
use tangram_ui as ui;

#[derive(Clone)]
pub struct TopbarProps {
	pub topbar_avatar: Option<TopbarAvatar>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TopbarAvatar {
	pub avatar_url: Option<String>,
}

#[component]
pub fn Topbar(topbar_avatar: Option<TopbarAvatar>) {
	let mut items = Vec::new();
	if let Some(topbar_avatar) = topbar_avatar {
		items.push(ui::TopbarItem {
			element: Some(html! {
				<ui::Link title={None} class={None} href={Some("/user".to_owned())}>
					<ui::Avatar src={topbar_avatar.avatar_url} />
				</ui::Link>
			}),
			href: "/user".to_owned(),
			title: "Settings".to_owned(),
		})
	}
	let logo = Some(html! {
		<Logo class={None} color={None} color_scheme={LogoScheme::Multi} />
	});
	html! {
		<ui::Topbar
			border={None}
			background_color={"var(--header-color)".to_owned()}
			dropdown_background_color={"var(--surface-color)".to_owned()}
			foreground_color={"var(--text-color)".to_owned()}
			items={Some(items)}
			logo={logo}
			logo_href={Some("/".to_owned())}
			logo_img_url={None}
			title={Some("tangram".to_owned())}
		/>
	}
}
