import { Children, PinwheelInfo, h } from '../deps'
import { Layout } from './layout'

type PageLayoutProps = {
	background?: boolean
	children?: Children
	info: PinwheelInfo
}

export function PageLayout(props: PageLayoutProps) {
	return (
		<Layout background={props.background} info={props.info}>
			<div class="page-layout-wrapper">{props.children}</div>
		</Layout>
	)
}
