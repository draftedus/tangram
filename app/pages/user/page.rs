use html::{component, html};
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::PageInfo,
};
use tangram_ui as ui;

pub struct Props {
	pub app_layout_info: AppLayoutInfo,
	pub inner: Inner,
}

pub enum Inner {
	Auth(AuthProps),
	NoAuth(NoAuthProps),
}

#[derive(Clone)]
pub struct AuthProps {
	pub email: String,
	pub organizations: Vec<Organization>,
	pub repos: Vec<Repo>,
}

#[derive(Clone)]
pub struct Organization {
	pub id: String,
	pub name: String,
}

pub struct NoAuthProps {
	pub repos: Vec<Repo>,
}

#[derive(Clone)]
pub struct Repo {
	pub id: String,
	pub title: String,
}

pub fn render(props: Props, page_info: PageInfo) -> String {
	let inner = match props.inner {
		Inner::Auth(inner) => {
			html! {
				<Auth props={inner} />
			}
		}
		Inner::NoAuth(_) => {
			html! {
				<NoAuth />
			}
		}
	};
	let html = html! {
		<AppLayout info={props.app_layout_info} page_info={page_info}>
			{inner}
		</AppLayout>
	};
	html.render_to_string()
}

#[component]
pub fn NoAuth() {
	html! {
		<ui::S1>
			<ui::P>
				{"You are using the free version of tangram that does not support user accounts or organizations. Checkout out the different plans that allow you to collaborate with your team."}
			</ui::P>
		</ui::S1>
	}
}

#[component]
pub fn Auth(props: AuthProps) {
	let repos_list_table = html! {
		<ui::Table width={"100%".to_owned()}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell
						text_align={None}
						expand={None}
						color={None}
					>
						{"Repo Title"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.repos.iter().map(|repo| html! {
					<ui::TableRow color={None}>
						<ui::TableCell color={None} expand={None}>
							<ui::Link
								class={None}
								title={None}
								href={format!("/repos/{}/", repo.id)}
							>
								{repo.title.clone()}
							</ui::Link>
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	};
	let organizations_table = html! {
		<ui::Table width={"100%".to_owned()}>
			<ui::TableHeader>
				<ui::TableRow color={None}>
					<ui::TableHeaderCell color={None} expand={None} text_align={None}>{"Organization Name"}</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.organizations.iter().map(|organization| html! {
					<ui::TableRow color={None}>
						<ui::TableCell expand={None} color={None}>
							<ui::Link
								href={format!("/organizations/{}/", organization.id)}
								title={None}
								class={None}
							>
								{organization.name.clone()}
							</ui::Link>
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	};
	html! {
		<ui::S1>
			<ui::SpaceBetween>
				<ui::H1 center={false}>{"User"}</ui::H1>
				<ui::Form
					id={None}
					autocomplete={None}
					enc_type={None}
					action={None}
					post={true}
				>
					<input
						name={"action"}
						type={"hidden"}
						value={"logout"}
					/>
					<ui::Button
						button_type={ui::ButtonType::Button}
						color={"var(--red)".to_owned()}
						disabled={None}
						download={None}
						href={None}
						id={None}
					>
						{"Logout"}
					</ui::Button>
				</ui::Form>
			</ui::SpaceBetween>
			<ui::S2>
				<ui::Form
					post={None}
					action={None}
					id={None}
					enc_type={None}
					autocomplete={None}
				>
					<ui::TextField
						required={None}
						autocomplete={None}
						disabled={None}
						label={"Email".to_owned()}
						readonly={true}
						name={None}
						placeholder={None}
						value={props.email}
					/>
				</ui::Form>
			</ui::S2>
			<ui::S2>
				<ui::SpaceBetween>
					<ui::H2 center={false}>{"User Repos"}</ui::H2>
					<ui::Button
						button_type={ui::ButtonType::Button}
						color={None}
						disabled={None}
						download={None}
						href={"/repos/new".to_owned()}
						id={None}
					>
						{"Create New Repo"}
					</ui::Button>
				</ui::SpaceBetween>
				{if !props.repos.is_empty() {
					repos_list_table
				} else {
					html! {
						<ui::Card>
							<ui::P>{"You do not have any repos."}</ui::P>
						</ui::Card>
					}
				}}
			</ui::S2>
			<ui::S2>
				<ui::SpaceBetween>
					<ui::H2 center={false}>{"Organizations"}</ui::H2>
					<ui::Button
						color={None}
						disabled={None}
						download={None}
						id={None}
						button_type={ui::ButtonType::Button}
						href={"/organizations/new".to_owned()}
					>
						{"Create New Organization"}
					</ui::Button>
				</ui::SpaceBetween>
				{if !props.organizations.is_empty() {
					organizations_table
				} else {
					html! {
						<ui::Card>
							<ui::P>{"You do not have any organizations."}</ui::P>
						</ui::Card>
					}
				}}
			</ui::S2>
		</ui::S1>
	}
}
