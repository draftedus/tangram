import { Children, h } from '../deps'
import { Layout } from './layout'

type PageLayoutProps = {
	background?: boolean
	children?: Children
}

export function PageLayout(props: PageLayoutProps) {
	return (
		<Layout background={props.background}>
			<div class="page-layout-wrapper">{props.children}</div>
		</Layout>
	)
}
