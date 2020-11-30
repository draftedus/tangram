use super::document::{Document, PageInfo};
use html::{component, html};
use tangram_app_common::logo::{Logo, LogoScheme};
use tangram_ui as ui;

#[component]
pub fn AuthLayout(page_info: PageInfo) {
	html! {
		<Document page_info={page_info}>
			<div class="auth-layout">
				<div class="auth-layout-logo-wrapper">
					<Logo class={None} color={None} color_scheme={LogoScheme::Multi} />
				</div>
				<div class="auth-layout-card-wrapper">
					<ui::Card>{children}</ui::Card>
				</div>
			</div>
		</Document>
	}
}
