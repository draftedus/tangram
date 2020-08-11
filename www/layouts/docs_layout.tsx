import { Children, PinwheelInfo, h, ui } from '../deps'
import { Layout } from './layout'

type DocsLayoutProps = {
	children?: Children
	info: PinwheelInfo
}

export function DocsLayout(props: DocsLayoutProps) {
	return (
		<Layout info={props.info}>
			<div class="docs-layout-wrapper">
				<PageNav />
				<div>{props.children}</div>
			</div>
		</Layout>
	)
}

function PageNav() {
	return (
		<ui.NestedNav>
			<ui.NestedNavItem href="/docs/">{'Home'}</ui.NestedNavItem>
			<ui.NestedNavSection>
				<ui.NestedNavSectionTitle>{'Install'}</ui.NestedNavSectionTitle>
				<ui.NestedNavItem href="/docs/install">{'Install'}</ui.NestedNavItem>
			</ui.NestedNavSection>
			<ui.NestedNavSection>
				<ui.NestedNavSectionTitle>{'Getting Started'}</ui.NestedNavSectionTitle>
				<ui.NestedNavItem href="/docs/getting-started/train">
					{'Train'}
				</ui.NestedNavItem>
				<ui.NestedNavItem href="/docs/getting-started/predict">
					{'Predict'}
				</ui.NestedNavItem>
				<ui.NestedNavItem href="/docs/getting-started/report">
					{'Report'}
				</ui.NestedNavItem>
				<ui.NestedNavItem href="/docs/getting-started/tune">
					{'Tune'}
				</ui.NestedNavItem>
				<ui.NestedNavItem href="/docs/getting-started/monitor">
					{'Monitor'}
				</ui.NestedNavItem>
			</ui.NestedNavSection>
			<ui.NestedNavSection>
				<ui.NestedNavSectionTitle>{'Advanced'}</ui.NestedNavSectionTitle>
				<ui.NestedNavItem href="/docs/on-prem">{'On-Prem'}</ui.NestedNavItem>
			</ui.NestedNavSection>
		</ui.NestedNav>
	)
}
