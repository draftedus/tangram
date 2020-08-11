import { Children, PinwheelInfo, h } from '../deps'
import { Layout } from './layout'

type PageLayoutProps = {
	background?: boolean
	children?: Children
	pinwheelInfo: PinwheelInfo
}

export function PageLayout(props: PageLayoutProps) {
	return (
		<Layout background={props.background} pinwheelInfo={props.pinwheelInfo}>
			<div class="page-layout-wrapper">{props.children}</div>
		</Layout>
	)
}
