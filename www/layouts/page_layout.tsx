import { Layout } from './layout'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import { ComponentChildren, h } from 'preact'

type PageLayoutProps = {
	background?: boolean
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function PageLayout(props: PageLayoutProps) {
	return (
		<Layout background={props.background} pinwheelInfo={props.pinwheelInfo}>
			<div class="page-layout-wrapper">{props.children}</div>
		</Layout>
	)
}
