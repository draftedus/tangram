import { Layout } from "./layout"
import "./page_layout.css"
import { PageInfo } from "@tangramhq/pinwheel"
import { ComponentChildren, h } from "preact"

type PageLayoutProps = {
	background?: boolean
	children?: ComponentChildren
	pageInfo: PageInfo
}

export function PageLayout(props: PageLayoutProps) {
	return (
		<Layout background={props.background} pageInfo={props.pageInfo}>
			<div class="page-layout-wrapper">{props.children}</div>
		</Layout>
	)
}
