import { Logo, LogoScheme } from '../../www/components/logo'
import { Children, css, h, ui, useCss } from 'deps'

type TopbarLayoutProps = { children?: Children }

let gridCss = css({
	[`.topbar-layout-grid`]: {
		color: ui.variables.colors.text,
		display: 'grid',
		grid: 'auto 1fr / auto',
		height: '100vh',
		overflowX: 'hidden',
	},
})

export function TopbarLayout(props: TopbarLayoutProps) {
	useCss(gridCss)
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
