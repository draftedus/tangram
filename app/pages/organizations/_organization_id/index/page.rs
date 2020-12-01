use html::html;
use tangram_app_common::organizations::Member;
use tangram_app_layouts::app_layout::{AppLayout, AppLayoutInfo};
use tangram_app_layouts::document::PageInfo;
use tangram_ui as ui;

pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub id: String,
	pub members: Vec<Member>,
	pub name: String,
	pub repos: Vec<Repo>,
	pub user_id: String,
}

pub struct Repo {
	pub id: String,
	pub title: String,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let html = html! {
		<AppLayout info={props.app_layout_info.clone()} page_info={page_info}>
			<ui::S1>
				<ui::H1 center={Some(false)}>{props.name.clone()}</ui::H1>
				<ui::S2>
					<ui::SpaceBetween>
						<ui::H2 center={Some(false)}>{"Details"}</ui::H2>
						<ui::Button
							color={Some("var(--gray)".to_owned())}
							button_type={ui::ButtonType::Button}
							download={None}
							disabled={Some(false)}
							href={Some(format!("/organizations/{}/edit", props.id))}
							id={None}
						>
							{"Edit"}
						</ui::Button>
					</ui::SpaceBetween>
					<ui::TextField
						autocomplete={None}
						disabled={Some(true)}
						name={None}
						placeholder={None}
						value={Some(props.name.clone())}
						required={None}
						label={Some("Organization Name".to_owned())}
						readonly={Some(true)}
					/>
					</ui::S2>
					<ui::S2>
						<ui::SpaceBetween>
							<ui::H2 center={Some(false)}>{"Repos"}</ui::H2>
							<ui::Button
								id={None}
								color={None}
								disabled={None}
								download={None}
								button_type={ui::ButtonType::Button}
								href={Some("/repos/new".to_owned())}
							>
								{"Create New Repo"}
							</ui::Button>
						</ui::SpaceBetween>
						{
							if !props.repos.is_empty() {
								html! {
									<ui::Table width={Some("100%".to_owned())}>
										<ui::TableHeader>
											<ui::TableRow color={None}>
												<ui::TableHeaderCell
													color={None}
													expand={None}
													text_align={None}
												>
													{"Repo Title"}
												</ui::TableHeaderCell>
											</ui::TableRow>
										</ui::TableHeader>
										<ui::TableBody>
										{props.repos.iter().map(|repo| {
											html! {
												<ui::TableRow color={None}>
													<ui::TableCell color={None} expand={None}>
														<ui::Link
															title={None}
															class={None}
															href={Some(format!("/repos/{}/", repo.id))}
														>
															{repo.title.clone()}
														</ui::Link>
													</ui::TableCell>
												</ui::TableRow>
											}
										}).collect::<Vec<_>>()}
										</ui::TableBody>
									</ui::Table>
								}
							} else {
								html! {
									<ui::Card>
										<ui::P>{"This organization does not have any repos."}</ui::P>
									</ui::Card>
								}
							}
						}
				</ui::S2>
				<ui::S2>
					<ui::SpaceBetween>
						<ui::H2 center={Some(false)}>{"Members"}</ui::H2>
						<ui::Button
							id={None}
							color={None}
							disabled={None}
							download={None}
							button_type={ui::ButtonType::Button}
							href={Some(format!("/organizations/{}/members/new", props.id))}
						>
							{"Invite Team Member"}
						</ui::Button>
					</ui::SpaceBetween>
					<ui::Table width={Some("100%".to_owned())}>
						<ui::TableHeader>
							<ui::TableRow color={None}>
							<ui::TableHeaderCell
								color={None}
								expand={None}
								text_align={None}
							>
								{"Email"}
							</ui::TableHeaderCell>
							<ui::TableHeaderCell
								color={None}
								expand={None}
								text_align={None}
							>
								{"Role"}
							</ui::TableHeaderCell>
							<ui::TableHeaderCell
								color={None}
								expand={None}
								text_align={None}
							>
								{"Remove"}
							</ui::TableHeaderCell>
							</ui::TableRow>
						</ui::TableHeader>
						<ui::TableBody>
						{props.members.iter().map(|member| html! {
							<ui::TableRow color={None}>
								<ui::TableCell color={None} expand={None}>{member.email.clone()}</ui::TableCell>
								<ui::TableCell color={None} expand={None}>
								{
									if member.is_admin {
										"Admin"
									} else {
										"Member"
									}
								}
								</ui::TableCell>
								<ui::TableCell color={None} expand={None}>
								{
									if props.user_id != member.id {
										Some(html! {
											<ui::Form
												id={None}
												post={Some(true)}
												action={None}
												autocomplete={None}
												enc_type={None}
											>
												<input
													name={Some("action".to_owned())}
													type={Some("hidden".to_owned())}
													value={Some("delete_member".to_owned())}
												/>
												<input
													name={Some("member_id".to_owned())}
													type={Some("hidden".to_owned())}
													value={member.id.clone()}
												/>
												<ui::Button
													href={None}
													download={None}
													id={None}
													disabled={None}
													button_type={ui::ButtonType::Submit}
													color={Some("var(--red)".to_owned())}
												>
													{"Remove"}
												</ui::Button>
											</ui::Form>
										})
									} else {
										None
									}
								}
								</ui::TableCell>
							</ui::TableRow>
						}).collect::<Vec<_>>()}
						</ui::TableBody>
					</ui::Table>
				</ui::S2>
				<ui::S2>
					<ui::H2 center={Some(false)}>{"Danger Zone"}</ui::H2>
					<ui::Form
						autocomplete={None}
						action={None}
						post={Some(true)}
						enc_type={None}
						id={None}
					>
					<input
						name={"action"}
						type="hidden"
						value="delete_organization"
					/>
					<ui::Button
						button_type={ui::ButtonType::Submit}
						disabled={None}
						href={None}
						download={None}
						id={None}
						color={Some("var(--red)".to_owned())}
					>
						{"Delete Organization"}
					</ui::Button>
					</ui::Form>
				</ui::S2>
			</ui::S1>
		</AppLayout>
	};
	html.render_to_string()
}
