use html::{component, html, style};
use tangram_app_layouts::app_layout::AppLayoutInfo;
use tangram_app_layouts::{app_layout::AppLayout, document::PageInfo};
use tangram_ui as ui;

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub repos_table: Vec<RepoTableItem>,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoTableItem {
	pub id: String,
	pub title: String,
	pub owner_name: Option<String>,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let html = html! {
		<AppLayout info={props.app_layout_info} page_info={page_info}>
			<ui::S1>
				<ui::SpaceBetween>
					<ui::H1 center={Some(false)}>{"Repositories"}</ui::H1>
					<ui::Button
						color={None}
						button_type={ui::ButtonType::Button}
						href={Some("/repos/new".to_owned())}
						disabled={None}
						download={None}
						id={None}
					>
						{"Create Repo"}
					</ui::Button>
				</ui::SpaceBetween>
				{
					if !props.repos_table.is_empty() {
						html! {
							<RepoListTable repos_table={props.repos_table}/>
						}
					} else {
						html! {
							<ui::Card>
								<ui::P>{"You do not have any repositories."}</ui::P>
							</ui::Card>
						}
					}
				}
			</ui::S1>
		</AppLayout>
	};
	html.render_to_string()
}

#[component]
pub fn RepoListTable(repos_table: Vec<RepoTableItem>) {
	let form_style = style! {
		"margin" => "0",
	};
	html! {
		<ui::Table width={Some("100%".to_owned())}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell
						text_align={None}
						color={None}
						expand={Some(true)}
					>
						{"Name"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell
						text_align={None}
						color={None}
						expand={None}
					>
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{repos_table.into_iter().map(|repo| html! {
					<ui::TableRow color={None}>
						<ui::TableCell color={None} expand={None}>
							<ui::Link
								class={None}
								title={None}
								href={Some(format!("/repos/{}/", repo.id))}>
								{
									repo.owner_name.map(|owner_name| {
										html! {
											<>
											{format!("{}/", owner_name)}
											</>
										}
									})
								}
								{repo.title}
							</ui::Link>
						</ui::TableCell>
						<ui::TableCell color={None} expand={None}>
							<form style={form_style.clone()} method="post">
								<input name="action" type="hidden" value="delete_repo" />
								<input name="repo_id" type="hidden" value={repo.id} />
								<ui::Button
									href={None}
									button_type={ui::ButtonType::Button}
									disabled={None}
									download={None}
									id={None}
									color={Some("var(--red)".to_owned())}
								>
									{"Delete"}
								</ui::Button>
							</form>
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}
