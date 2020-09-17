import './app_layout.css'
import { Layout } from './topbar_layout'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import { ComponentChildren, h } from 'preact'

type AppLayoutProps = {
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function AppLayout(props: AppLayoutProps) {
	return (
		<Layout pinwheelInfo={props.pinwheelInfo}>
			<div class="app-layout">{props.children}</div>
		</Layout>
	)
}
