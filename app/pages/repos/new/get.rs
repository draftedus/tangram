use super::props::{Owner, Props};
use super::render::render;
use tangram_app_common::{
	error::{redirect_to_login, service_unavailable},
	user::{authorize_user, User},
	Context,
};
use tangram_app_layouts::app_layout::get_app_layout_info;
use tangram_deps::{http, hyper, pinwheel::Pinwheel, sqlx, sqlx::prelude::*};
use tangram_util::error::Result;

pub async fn get(
	_pinwheel: &Pinwheel,
	context: &Context,
	request: http::Request<hyper::Body>,
) -> Result<http::Response<hyper::Body>> {
	let mut db = match context.pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(&request, &mut db, context.options.auth_enabled).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let app_layout_info = get_app_layout_info(context).await?;
	let owners = match user {
		User::Root => None,
		User::Normal(user) => {
			let mut owners = vec![Owner {
				value: format!("user:{}", user.id),
				title: user.email,
			}];
			let rows = sqlx::query(
				"
				select
					organizations.id,
					organizations.name
				from organizations
				join organizations_users
					on organizations_users.organization_id = organizations.id
					and organizations_users.user_id = $1
			",
			)
			.bind(&user.id.to_string())
			.fetch_all(&mut *db)
			.await?;
			for row in rows {
				let id: String = row.get(0);
				let title: String = row.get(1);
				owners.push(Owner {
					value: format!("organization:{}", id),
					title,
				})
			}
			Some(owners)
		}
	};
	let props = Props {
		app_layout_info,
		owners,
		error: None,
		owner: None,
		title: None,
	};
	let html = render(props).await?;
	db.commit().await?;
	let response = http::Response::builder()
		.status(http::StatusCode::OK)
		.body(hyper::Body::from(html))
		.unwrap();
	Ok(response)
}
