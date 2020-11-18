use tangram_app_common::{
	error::{bad_request, not_found, service_unavailable, unauthorized},
	user::{authorize_normal_user, authorize_normal_user_for_organization},
	Context,
};
use tangram_deps::{http, hyper, pinwheel::Pinwheel, serde_urlencoded, sqlx};
use tangram_util::{error::Result, id::Id};

#[derive(serde::Deserialize)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_organization")]
	DeleteOrganization,
	#[serde(rename = "delete_member")]
	DeleteMember(DeleteMemberAction),
}

#[derive(serde::Deserialize)]
struct DeleteMemberAction {
	member_id: String,
}

pub async fn post(
	_pinwheel: &Pinwheel,
	context: &Context,
	mut request: http::Request<hyper::Body>,
	organization_id: &str,
) -> Result<http::Response<hyper::Body>> {
	if !context.options.auth_enabled {
		return Ok(not_found());
	}
	let data = match hyper::body::to_bytes(request.body_mut()).await {
		Ok(data) => data,
		Err(_) => return Ok(bad_request()),
	};
	let action: Action = match serde_urlencoded::from_bytes(&data) {
		Ok(action) => action,
		Err(_) => return Ok(bad_request()),
	};
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_normal_user(&request, &mut db).await? {
		Ok(user) => user,
		Err(_) => return Ok(unauthorized()),
	};
	let organization_id: Id = match organization_id.parse() {
		Ok(organization_id) => organization_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_normal_user_for_organization(&mut db, &user, organization_id).await? {
		return Ok(not_found());
	}
	let response = match action {
		Action::DeleteOrganization => delete_organization(&mut db, organization_id).await?,
		Action::DeleteMember(action) => delete_member(&mut db, organization_id, action).await?,
	};
	db.commit().await?;
	Ok(response)
}

async fn delete_organization(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
) -> Result<http::Response<hyper::Body>> {
	sqlx::query(
		"
		delete from organizations
		where
			id = $1
	",
	)
	.bind(&organization_id.to_string())
	.execute(&mut *db)
	.await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, "/user")
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}

async fn delete_member(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	action: DeleteMemberAction,
) -> Result<http::Response<hyper::Body>> {
	let DeleteMemberAction { member_id } = action;
	let member_id: Id = match member_id.parse() {
		Ok(member_id) => member_id,
		Err(_) => return Ok(not_found()),
	};
	sqlx::query(
		"
		delete from organizations_users
		where
			organization_id = $1
			and user_id = $2
	",
	)
	.bind(&organization_id.to_string())
	.bind(&member_id.to_string())
	.execute(&mut *db)
	.await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, "/user")
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}
