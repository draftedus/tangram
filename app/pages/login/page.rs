use html::html;
use tangram_app_layouts::auth_layout::AuthLayout;
use tangram_app_layouts::document::PageInfo;
use tangram_ui as ui;

#[derive(serde::Serialize)]
pub struct Props {
	pub code: bool,
	pub email: Option<String>,
	pub error: Option<String>,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let html = html! {
		<AuthLayout page_info={page_info}>
			<ui::Form
				action={None}
				autocomplete={None}
				post={Some(true)}
				id={None}
				enc_type={None}
			>
				{
					props.error.map(|error| html! {
						<ui::Alert
							title={None}
							level={ui::Level::Danger}
						>
							{error}
						</ui::Alert>
					})
				}
				<ui::TextField
					autocomplete={Some("username".to_owned())}
					disabled={Some(props.email.is_none())}
					name={Some("email".into())}
					placeholder={Some("Email".to_owned())}
					value={props.email}
					required={None}
					label={None}
					readonly={None}
				/>
				<ui::Button
					download={None}
					id={None}
					href={None}
					color={None}
					disabled={None}
					button_type={ui::ButtonType::Button}
				/>
				{
					if props.code {
						Some(html! {
							<div class="login-code-message">
								{"We emailed you a code. Copy and paste it above to log in."}
							</div>
						})
					} else {
						None
					}
				}
			</ui::Form>
		</AuthLayout>
	};
	html.render_to_string()
}
