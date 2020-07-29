use super::super::super::layouts::AuthLayout;
use crate::app::Context;
use anyhow::Result;
use html::{component, html};
use hyper::{Body, Request, Response, StatusCode};
use std::{collections::BTreeMap, sync::Arc};

#[derive(serde::Serialize)]
struct LoginProps {
	code: bool,
	email: Option<String>,
	error: Option<String>,
}

pub async fn page(
	_request: Request<Body>,
	context: Arc<Context>,
	search_params: Option<BTreeMap<String, String>>,
) -> Result<Response<Body>> {
	let email = search_params.as_ref().and_then(|s| s.get("email").cloned());
	let props = LoginProps {
		code: email.is_some(),
		error: None,
		email,
	};
	let html = context.pinwheel.render("/login", props).await?;
	// let html = html!(<Login email={None} />).render_to_string();
	Ok(Response::builder()
		.status(StatusCode::OK)
		.body(Body::from(html))
		.unwrap())
}

#[derive(PartialEq)]
enum Stage {
	Email,
	Code,
}

#[component]
fn Login(email: Option<String>) {
	let stage = if email.is_none() {
		Stage::Email
	} else {
		Stage::Code
	};
	let action = if stage == Stage::Email {
		"/login/actions/code"
	} else {
		"/login/actions/login"
	};
	html!(
		<AuthLayout>
			<form method="post" action={action}>
				<input
					type="text"
					autocomplete="username"
					placeholder="Email"
					name="email"
					disabled={email.is_some()}
					value={email.clone()}
				/>
				{if email.is_some() {
					Some(html!(
						<input type="text" name="code" placeholder="Code" />
					))
				} else {
					None
				}}
				<button>"Login"</button>
			</form>
		</AuthLayout>
	)
}
