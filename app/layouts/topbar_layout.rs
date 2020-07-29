use html::{component, html};

#[component]
pub fn TopbarLayout() {
	html!(
		<div class="topbar-layout-grid">
			<Topbar />
			<div>{props.children}</div>
		</div>
	)
}

#[component]
fn Topbar() {
	html!(
		<ui::Topbar
			activeTextColor={ui.colors.blue}
			backgroundColor={ui.variables.colors.header}
			dropdownBackgroundColor={ui.variables.colors.surface}
			foregroundColor={ui.variables.colors.text}
			items={vec![
				ui::TopbarItem {
					element: html!(
						<ui::Link page="/user/">
							<ui::Avatar />
						</ui::Link>
					),
					href: "/user/",
					title: "Settings",
				},
			]}
			logo={html!(<ui::Logo colorScheme={LogoScheme.Multi} />)}
			logoHref="/"
			menuSeparatorColor={ui.variables.colors.mutedText}
			title="tangram"
		/>
	)
}
