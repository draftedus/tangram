import { Layout } from './topbar_layout'
import { Children, PinwheelInfo, h } from 'deps'

type AppLayoutProps = {
	children?: Children
	pinwheelInfo: PinwheelInfo
}

export function AppLayout(props: AppLayoutProps) {
	return (
		<Layout pinwheelInfo={props.pinwheelInfo}>
			<div class="app-layout">{props.children}</div>
		</Layout>
	)
}
