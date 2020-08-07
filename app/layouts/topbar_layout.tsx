import { Logo, LogoScheme } from '../../www/components/logo'
import { Children, h, ui } from 'deps'

type TopbarLayoutProps = { children?: Children }

export function TopbarLayout(props: TopbarLayoutProps) {
	return (
		<div class="topbar-layout-grid">
			<Topbar />
			<div>{props.children}</div>
		</div>
	)
}

function Topbar() {
	return (
		<ui.Topbar
			activeTextColor={ui.colors.blue}
			backgroundColor={ui.variables.colors.header}
			dropdownBackgroundColor={ui.variables.colors.surface}
			foregroundColor={ui.variables.colors.text}
			items={[
				{
					element: (
						<ui.Link href="/user/">
							<ui.Avatar />
						</ui.Link>
					),
					href: '/user/',
					title: 'Settings',
				},
			]}
			logo={<Logo colorScheme={LogoScheme.Multi} />}
			logoHref="/"
			menuSeparatorColor={ui.variables.colors.mutedText}
			title="tangram"
		/>
	)
}
