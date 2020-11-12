// use html::{component, html};
// use tangram_ui as ui;

// #[derive(Clone)]
// pub struct TopbarAvatar {
// 	avatar_url: Option<String>,
// }

// #[component]
// fn Topbar(topbar_avatar: Option<TopbarAvatar>) {
// 	let mut items = vec![];
// 	if let Some(topbar_avatar) = topbar_avatar {
// 		items.push(ui::TopbarItem {
// 			element: html!(
// 				<ui::Link href="/user">
// 					<ui::Avatar src={topbar_avatar.avatar_url} />
// 				</ui::Link>
// 			),
// 			href: "/user",
// 			title: "Settings",
// 		})
// 	}
// 	html! {
// 		<ui::Topbar
// 			background_color="var(--header-color)"
// 			dropdown_background_color="var(--surface-color)"
// 			foreground_color="var(--text-color)"
// 			items={items}
// 			logo={html! { <Logo colorScheme={LogoScheme.Multi} /> }}
// 			logo_href="/"
// 			title="tangram"
// 		/>
// 	}
// }
