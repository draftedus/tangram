use super::document::{Document, PageInfo};
use html::{component, html};
use tangram_app_common::{
	topbar::{Topbar, TopbarAvatar},
	Context,
};
use tangram_util::error::Result;

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppLayoutInfo {
	topbar_avatar: Option<TopbarAvatar>,
}

pub async fn get_app_layout_info(context: &Context) -> Result<AppLayoutInfo> {
	let topbar_avatar = if context.options.auth_enabled {
		Some(TopbarAvatar { avatar_url: None })
	} else {
		None
	};
	Ok(AppLayoutInfo { topbar_avatar })
}

#[component]
pub fn AppLayout(page_info: PageInfo, info: AppLayoutInfo) {
	html! {
		<Document page_info={page_info}>
			<div class="app-layout-topbar-grid">
				<Topbar
					topbar_avatar={
						info.topbar_avatar.map(|topbar_avatar| TopbarAvatar {
							avatar_url: topbar_avatar.avatar_url,
						})
					}
				/>
				<div class="app-layout">{children}</div>
			</div>
		</Document>
	}
}
