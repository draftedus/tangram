import { Children, PinwheelInfo, h, ui } from '../deps'
import { Layout } from './layout'

type DocsLayoutProps = {
	children?: Children
	pagename: string
	pinwheelInfo: PinwheelInfo
}

export function DocsLayout(props: DocsLayoutProps) {
	return (
		<Layout pinwheelInfo={props.pinwheelInfo}>
			<div class="docs-layout-wrapper">
				<PageNav pagename={props.pagename} />
				<div>{props.children}</div>
			</div>
		</Layout>
	)
}

type PageNavProps = {
	pagename: string
}

function PageNav(props: PageNavProps) {
	return (
		<ui.NestedNav>
			<ui.NestedNavItem href="/docs/" selected={props.pagename === '/docs/'}>
				{'Home'}
			</ui.NestedNavItem>
			<ui.NestedNavSection>
				<ui.NestedNavSectionTitle>{'Install'}</ui.NestedNavSectionTitle>
				<ui.NestedNavItem
					href="/docs/install"
					selected={props.pagename === '/docs/install'}
				>
					{'Install'}
				</ui.NestedNavItem>
			</ui.NestedNavSection>
			<ui.NestedNavSection>
				<ui.NestedNavSectionTitle>{'Getting Started'}</ui.NestedNavSectionTitle>
				<ui.NestedNavItem
					href="/docs/getting-started/train"
					selected={props.pagename === '/docs/getting-started/train'}
				>
					{'Train'}
				</ui.NestedNavItem>
				<ui.NestedNavItem
					href="/docs/getting-started/predict"
					selected={props.pagename === '/docs/getting-started/predict'}
				>
					{'Predict'}
				</ui.NestedNavItem>
				<ui.NestedNavItem
					href="/docs/getting-started/report"
					selected={props.pagename === '/docs/getting-started/report'}
				>
					{'Report'}
				</ui.NestedNavItem>
				<ui.NestedNavItem
					href="/docs/getting-started/tune"
					selected={props.pagename === '/docs/getting-started/tune'}
				>
					{'Tune'}
				</ui.NestedNavItem>
				<ui.NestedNavItem
					href="/docs/getting-started/monitor"
					selected={props.pagename === '/docs/getting-started/monitor'}
				>
					{'Monitor'}
				</ui.NestedNavItem>
			</ui.NestedNavSection>
		</ui.NestedNav>
	)
}
