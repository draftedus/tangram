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
			activeTextColor="var(--blue)"
			backgroundColor="var(--header-color)"
			dropdownBackgroundColor="var(--surface-color)"
			foregroundColor="var(--text-color)"
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
			menuSeparatorColor="var(--muted-text-color)"
			title="tangram"
		/>
	)
}
