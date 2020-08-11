import { TopbarLayout } from './topbar_layout'
import { Children, PinwheelInfo, h } from 'deps'

type AppLayoutProps = {
	children?: Children
	info: PinwheelInfo
}

export function AppLayout(props: AppLayoutProps) {
	return (
		<TopbarLayout info={props.info}>
			<div class="app-layout">{props.children}</div>
		</TopbarLayout>
	)
}
