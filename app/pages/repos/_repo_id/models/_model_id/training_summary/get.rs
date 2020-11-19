use super::props::Props;
use html::html;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	model::get_model,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_layouts::model_layout::get_model_layout_info;
use tangram_app_layouts::{
	document::PageInfo,
	model_layout::{ModelLayout, ModelSideNavItem},
};
use tangram_deps::{http, hyper, pinwheel::client};
use tangram_ui as ui;
use tangram_util::error::Result;
use tangram_util::id::Id;

pub async fn get(
	context: &Context,
	request: http::Request<hyper::Body>,
	model_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let model = get_model(&mut db, model_id).await?;
	let model_layout_info = get_model_layout_info(&mut db, context, model_id).await?;
	let page_info = PageInfo {
		client_wasm_js_src: Some(client!("client/Cargo.toml")),
	};
	let props = Props {
		id: model_id.to_string(),
		// inner,
		model_layout_info,
	};
	let comparison_metric = "auc_roc";
	db.commit().await?;
	let html = html! {
		<ModelLayout page_info={page_info} info={props.model_layout_info} selected_item={ModelSideNavItem::TrainingSummary}>
			<ui::S1>
				<ui::H1 center={None}>{"Training Summary"}</ui::H1>
				<ui::S2>
					<ui::Table width={Some("100%".to_owned())}>
						<ui::TableHeader>
							<ui::TableRow color={None}>
								<ui::TableHeaderCell color={None} text_align={None} expand={None}>
									"Model Number"
								</ui::TableHeaderCell>
								<ui::TableHeaderCell color={None} text_align={None} expand={None}>
									"Model Type"
								</ui::TableHeaderCell>
								<ui::TableHeaderCell color={None} text_align={None} expand={None}>
									{comparison_metric}
								</ui::TableHeaderCell>
							</ui::TableRow>
						</ui::TableHeader>
						<ui::TableRow color={None}>
							<ui::TableCell color={None}>
								"1"
							</ui::TableCell>
							<ui::TableCell color={None}>
								"gradient boosted decision tree"
							</ui::TableCell>
							<ui::TableCell color={None}>
								"87.7"
							</ui::TableCell>
						</ui::TableRow>
						<ui::TableRow color={None}>
							<ui::TableCell color={None}>
								"2"
							</ui::TableCell>
							<ui::TableCell color={None}>
								"linear model"
							</ui::TableCell>
							<ui::TableCell color={None}>
								"83.7"
							</ui::TableCell>
						</ui::TableRow>
					</ui::Table>
				</ui::S2>
			</ui::S1>
		</ModelLayout>
	};
	let body = html.render_to_string();
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(body))
		.unwrap();
	Ok(response)
}
