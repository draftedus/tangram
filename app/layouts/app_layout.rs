use super::document::{Document, PageInfo};
use html::{component, html};
use tangram_app_common::Context;
use tangram_util::error::Result;

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppLayoutInfo {
	topbar_avatar: Option<TopbarAvatar>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct TopbarAvatar {
	avatar_url: Option<String>,
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
pub fn AppLayout(page_info: PageInfo, _info: AppLayoutInfo) {
	html! {
		<Document page_info={page_info}>
			<div class="app-layout-topbar-grid">
				// <Topbar topbarAvatar={info.topbar_avatar} />
				<div class="app-layout">{children}</div>
			</div>
		</Document>
	}
}
