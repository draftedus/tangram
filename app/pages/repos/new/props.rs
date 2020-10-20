use crate::common::user::User;
use anyhow::Result;
use sqlx::prelude::*;

#[derive(serde::Serialize)]
pub struct Props {
	error: Option<String>,
	title: Option<String>,
	owner: Option<String>,
	owners: Option<Vec<Owner>>,
}

#[derive(serde::Serialize)]
pub struct Owner {
	value: String,
	title: String,
}

pub async fn props(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	user: Option<User>,
	error: Option<String>,
	title: Option<String>,
	owner: Option<String>,
) -> Result<Props> {
	if let Some(user) = user {
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
					and organizations_users.user_id = ?1
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
		Ok(Props {
			owners: Some(owners),
			error,
			owner,
			title,
		})
	} else {
		Ok(Props {
			owners: None,
			error,
			owner,
			title,
		})
	}
}
