use super::document::{Document, PageInfo};
use html::{component, html};
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	repos::get_model_version_ids,
	topbar,
	// <<<<<<< Updated upstream
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_deps::{base64, http, hyper, pinwheel::Pinwheel, sqlx, sqlx::prelude::*};
use tangram_ui as ui;
use tangram_util::{error::Result, id::Id};

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelLayoutInfo {
	pub model_id: Id,
	pub model_version_ids: Vec<Id>,
	pub owner: Option<Owner>,
	pub repo_id: String,
	pub repo_title: String,
	pub topbar_avatar: Option<TopbarAvatar>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TopbarAvatar {
	avatar_url: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type", content = "value")]
pub enum Owner {
	#[serde(rename = "user")]
	User { id: Id, email: String },
	#[serde(rename = "organization")]
	Organization { id: Id, name: String },
}

pub async fn get_model_layout_info(
	mut db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	model_id: Id,
) -> Result<ModelLayoutInfo> {
	let row = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				repos.user_id,
				users.email,
				repos.organization_id,
				organizations.name
			from repos
			join models
				on models.repo_id = repos.id
			left join users
				on users.id = repos.user_id
			left join organizations
				on organizations.id = repos.organization_id
			where models.id = $1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let repo_id: String = row.get(0);
	let repo_id: Id = repo_id.parse()?;
	let repo_title: String = row.get(1);
	let model_version_ids = get_model_version_ids(&mut db, repo_id).await?;
	let owner_organization_id: Option<String> = row.get(2);
	let owner_organization_name: Option<String> = row.get(3);
	let owner_user_id: Option<String> = row.get(4);
	let owner_user_email: Option<String> = row.get(5);
	let owner = if let Some(owner_user_id) = owner_user_id {
		Some(Owner::User {
			id: owner_user_id.parse().unwrap(),
			email: owner_user_email.unwrap(),
		})
	} else if let Some(owner_organization_id) = owner_organization_id {
		Some(Owner::Organization {
			id: owner_organization_id.parse().unwrap(),
			name: owner_organization_name.unwrap(),
		})
	} else {
		None
	};
	let topbar_avatar = if context.options.auth_enabled {
		Some(TopbarAvatar { avatar_url: None })
	} else {
		None
	};
	Ok(ModelLayoutInfo {
		model_id,
		model_version_ids,
		owner,
		repo_id: repo_id.to_string(),
		repo_title,
		topbar_avatar,
	})
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_model")]
	DeleteModel,
	#[serde(rename = "download_model")]
	DownloadModel,
}

pub async fn post(
	_pinwheel: &Pinwheel,
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
	let response = delete_model(&mut db, model_id).await?;
	db.commit().await?;
	Ok(response)
}

async fn delete_model(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
) -> Result<http::Response<hyper::Body>> {
	sqlx::query(
		"
		delete from models
		where
			models.id = $1
	",
	)
	.bind(&model_id.to_string())
	.execute(&mut *db)
	.await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, "/")
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}

pub async fn download(
	_pinwheel: &Pinwheel,
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
	let row = sqlx::query(
		"
		select
			data
		from models
		where
			models.id = $1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let data: String = row.get(0);
	let data = base64::decode(data)?;
	db.commit().await?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(data))
		.unwrap();
	Ok(response)
}

#[derive(Clone, PartialEq)]
pub enum ModelSideNavItem {
	Overview,
	TrainingSummary,
	TrainingStats,
	TrainingMetrics,
	TrainingImportances,
	Prediction,
	Tuning,
	ProductionPredictions,
	ProductionStats,
	ProductionMetrics,
}

#[component]
pub fn ModelLayout(info: ModelLayoutInfo, page_info: PageInfo, selected_item: ModelSideNavItem) {
	let selected_model_version_id = info
		.model_version_ids
		.iter()
		.find(|model_version_id| *model_version_id == &info.model_id)
		.unwrap();
	html! {
		<Document page_info={page_info}>
			<div class="model-layout-topbar-grid">
				<topbar::Topbar
					topbar_avatar={
						info.topbar_avatar.as_ref().map(|topbar_avatar| topbar::TopbarAvatar {
							avatar_url: topbar_avatar.avatar_url.to_owned(),
						})
					}
				/>
				<div class="model-layout">
					<ModelLayoutTopbar
						model_layout_info={info.clone()}
						selected_model_version_id={selected_model_version_id.to_string()}
					/>
					<div class="model-layout-grid">
						<div class="model-layout-side-nav-wrapper">
							<ModelSideNav
								id={info.model_id.to_string()}
								repo_title={info.repo_title}
								selected_item={selected_item}
							/>
						</div>
						<div class="model-layout-content">{children}</div>
					</div>
				</div>
			</div>
		</Document>
	}
}

#[component]
pub fn ModelLayoutTopbar(model_layout_info: ModelLayoutInfo, selected_model_version_id: String) {
	struct OwnerInfo {
		title: String,
		url: String,
	}
	let owner = model_layout_info.owner.clone();
	let owner_info = owner.map(|owner| match owner {
		Owner::Organization { name, id } => OwnerInfo {
			title: name,
			url: format!("/organizations/{}", id),
		},
		Owner::User { email, .. } => OwnerInfo {
			title: email,
			url: "/user".to_owned(),
		},
	});
	html! {
		<div class="model-layout-topbar">
			<div class="model-layout-owner-slash-repo-slash-model-wrapper">
				<div class="model-layout-owner-slash-repo-wrapper">
					{owner_info.map(|owner_info| {
						html! {
							<>
								<a
									class="model-layout-owner-slash-repo-link"
									href={owner_info.url}
									title="owner"
								>
									{owner_info.title}
								</a>
								<span class="model-layout-owner-slash-repo-slash">{"/"}</span>
							</>
						}
					})
				}
				<a
					class="model-layout-owner-slash-repo-link"
					href={format!("/repos/{}/", model_layout_info.repo_id)}
					title="repo"
				>
					{model_layout_info.repo_title.clone()}
				</a>
			</div>
		</div>
		<div class="model-layout-topbar-actions-wrapper">
			<div class="model-layout-topbar-version-select-wrapper">
				<ui::Details
					options={
						Some(model_layout_info.model_version_ids.iter().map(|model_version_id| ui::DetailsOption {
							href: format!("/repos/{}/", model_layout_info.repo_id.clone()),
							title: model_version_id.to_string(),
						}).collect::<Vec<_>>())
					}
					summary={Some(format!("Version: {}", selected_model_version_id))}
				/>
			</div>
			<ui::Button
				button_type={ui::ButtonType::Button}
				disabled={None}
				id={None}
				download={Some(format!("{}.tangram", model_layout_info.repo_title))}
				href={Some(format!("/repos/{}/models/{}/download", model_layout_info.repo_id, model_layout_info.model_id))}
			>
				{"Download"}
			</ui::Button>
			<ui::Button
				button_type={ui::ButtonType::Button}
				disabled={None}
				id={None}
				download={None}
				href={Some(format!("/repos/{}/models/new", model_layout_info.repo_id))}>
				{"Upload New Version"}
			</ui::Button>
		</div>
	</div>
	}
}

#[component]
pub fn ModelSideNav(id: String, repo_title: String, selected_item: ModelSideNavItem) {
	html! {
		<ui::SideNav>
			<ui::SideNavSection>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/", id, id)}
					selected={Some(selected_item == ModelSideNavItem::Overview)}
				>
					{"Overview"}
				</ui::SideNavItem>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/training_summary", id, id)}
					selected={Some(selected_item == ModelSideNavItem::TrainingSummary)}
				>
					{"Training Summary"}
				</ui::SideNavItem>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/training_stats/", id, id)}
					selected={Some(selected_item == ModelSideNavItem::TrainingStats)}
				>
					{"Training Stats"}
				</ui::SideNavItem>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/training_metrics/", id, id)}
					selected={Some(selected_item == ModelSideNavItem::TrainingMetrics)}
				>
					{"Training Metrics"}
				</ui::SideNavItem>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/training_importances", id, id)}
					selected={Some(selected_item == ModelSideNavItem::TrainingImportances)}
				>
					{"Training Importances"}
				</ui::SideNavItem>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/prediction", id, id)}
					selected={Some(selected_item == ModelSideNavItem::Prediction)}
				>
					{"Prediction"}
				</ui::SideNavItem>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/tuning", id, id)}
					selected={Some(selected_item == ModelSideNavItem::Tuning)}
				>
					{"Tuning"}
				</ui::SideNavItem>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/production_predictions/", id, id)}
					selected={Some(
						selected_item == ModelSideNavItem::ProductionPredictions
					)}
				>
					{"Production Predictions"}
				</ui::SideNavItem>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/production_stats/", id, id)}
					selected={Some(selected_item == ModelSideNavItem::ProductionStats)}
				>
					{"Production Stats"}
				</ui::SideNavItem>
				<ui::SideNavItem
					href={format!("/repos/{}/models/{}/production_metrics/", id, id)}
					selected={Some(selected_item == ModelSideNavItem::ProductionMetrics)}
				>
					{"Production Metrics"}
				</ui::SideNavItem>
			</ui::SideNavSection>
		</ui::SideNav>
	}
}
