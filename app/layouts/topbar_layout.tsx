import { Document, Logo, LogoScheme } from '../../www/layouts/layout'
import { ComponentChildren, PinwheelInfo, h, ui } from 'deps'

type LayoutProps = {
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function Layout(props: LayoutProps) {
	return (
		<Document pinwheelInfo={props.pinwheelInfo}>
			<div class="topbar-layout-grid">
				<Topbar />
				<div>{props.children}</div>
			</div>
		</Document>
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
						<ui.Link href="/user">
							<ui.Avatar />
						</ui.Link>
					),
					href: '/user',
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
