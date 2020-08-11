import { PinwheelInfo } from '../../ui/deps'
import { Logo, LogoScheme } from '../../www/components/logo'
import { Document } from '../../www/layouts/layout'
import { Children, h, ui } from 'deps'

type TopbarLayoutProps = {
	children?: Children
	pinwheelInfo: PinwheelInfo
}

export function TopbarLayout(props: TopbarLayoutProps) {
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
