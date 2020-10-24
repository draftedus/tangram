import './topbar_layout.css'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { Document, Logo, LogoScheme } from '@tangramhq/www'
import { ComponentChildren, h } from 'preact'

type LayoutProps = {
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
	// topbarAvatar: TopbarAvatar | null
}

export function TopbarLayout(props: LayoutProps) {
	return (
		<Document pinwheelInfo={props.pinwheelInfo}>
			<div class="topbar-layout-grid">
				<Topbar topbarAvatar={null} />
				<div>{props.children}</div>
			</div>
		</Document>
	)
}

type TopbarProps = {
	topbarAvatar: TopbarAvatar | null
}

export type TopbarAvatar = {
	avatarUrl: string | null
}

function Topbar(props: TopbarProps) {
	let items = []
	if (props.topbarAvatar) {
		items.push({
			element: (
				<ui.Link href="/user">
					<ui.Avatar src={props.topbarAvatar.avatarUrl} />
				</ui.Link>
			),
			href: '/user',
			title: 'Settings',
		})
	}
	return (
		<ui.Topbar
			activeTextColor="var(--blue)"
			backgroundColor="var(--header-color)"
			dropdownBackgroundColor="var(--surface-color)"
			foregroundColor="var(--text-color)"
			items={items}
			logo={<Logo colorScheme={LogoScheme.Multi} />}
			logoHref="/"
			menuSeparatorColor="var(--muted-text-color)"
			title="tangram"
		/>
	)
}
