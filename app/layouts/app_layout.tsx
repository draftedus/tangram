import './app_layout.css'
import { TopbarLayout } from './topbar_layout'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import { ComponentChildren, h } from 'preact'

type AppLayoutProps = {
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
	// topbarAvatar: TopbarAvatar | null
}

export function AppLayout(props: AppLayoutProps) {
	return (
		<TopbarLayout
			pinwheelInfo={props.pinwheelInfo}
			// topbarAvatar={props.topbarAvatar}
		>
			<div class="app-layout">{props.children}</div>
		</TopbarLayout>
	)
}
