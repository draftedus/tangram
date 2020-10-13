import './topbar_layout.css'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { Document, Logo, LogoScheme } from '@tangramhq/www'
import { ComponentChildren, h } from 'preact'

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
