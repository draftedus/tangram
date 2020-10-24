import { Layout } from './layout'
import './page_layout.css'
import { ComponentChildren, h } from 'preact'

type PageLayoutProps = {
	background?: boolean
	children?: ComponentChildren
	clientJsSrc?: string
	cssSrcs?: string[]
	preloadJsSrcs?: string[]
}

export function PageLayout(props: PageLayoutProps) {
	return (
		<Layout
			background={props.background}
			clientJsSrc={props.clientJsSrc}
			cssSrcs={props.cssSrcs}
			preloadJsSrcs={props.preloadJsSrcs}
		>
			<div class="page-layout-wrapper">{props.children}</div>
		</Layout>
	)
}
