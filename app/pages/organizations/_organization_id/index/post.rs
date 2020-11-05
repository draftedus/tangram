use crate::{
	common::{
		error::Error,
		user::{authorize_normal_user, authorize_normal_user_for_organization},
	},
	Context,
};
use hyper::{body::to_bytes, header, Body, Request, Response, StatusCode};
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
	context: &Context,
	mut request: Request<Body>,
	organization_id: &str,
) -> Result<Response<Body>> {
	if !context.options.auth_enabled {
		return Err(Error::NotFound.into());
	}
	let data = to_bytes(request.body_mut())
		.await
		.map_err(|_| Error::BadRequest)?;
	let action: Action = serde_urlencoded::from_bytes(&data).map_err(|_| Error::BadRequest)?;
	let mut db = context
		.pool
		.begin()
		.await
		.map_err(|_| Error::ServiceUnavailable)?;
	let user = authorize_normal_user(&request, &mut db)
		.await?
		.map_err(|_| Error::Unauthorized)?;
	let organization_id: Id = organization_id.parse().map_err(|_| Error::NotFound)?;
	if !authorize_normal_user_for_organization(&mut db, &user, organization_id).await? {
		return Err(Error::NotFound.into());
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
) -> Result<Response<Body>> {
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
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/user")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}

async fn delete_member(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	organization_id: Id,
	action: DeleteMemberAction,
) -> Result<Response<Body>> {
	let DeleteMemberAction { member_id } = action;
	let member_id: Id = member_id.parse().map_err(|_| Error::NotFound)?;
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
	let response = Response::builder()
		.status(StatusCode::SEE_OTHER)
		.header(header::LOCATION, "/user")
		.body(Body::empty())
		.unwrap();
	Ok(response)
}
