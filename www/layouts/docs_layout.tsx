import { Children, css, h, ui, useCss } from '../deps'
import { Layout } from './layout'

type DocsLayoutProps = { children?: Children }

let wrapperCss = css({
	[`.docs-layout-wrapper`]: {
		alignContent: 'start',
		alignItems: 'start',
		boxSizing: 'border-box',
		display: 'grid',
		margin: '0 auto',
		maxWidth: ui.variables.width.max,
		padding: '2rem 1rem',
		width: '100%',
	},
	[ui.mobile]: {
		[`.docs-layout-wrapper`]: {
			grid: '"page-nav" auto "content" auto / minmax(0, 1fr)',
			gridGap: '2rem',
		},
	},
	[ui.desktop]: {
		[`.docs-layout-wrapper`]: {
			grid: '"page-nav content" auto / 200px minmax(0, 1fr)',
			gridGap: '1rem',
		},
	},
})

export function DocsLayout(props: DocsLayoutProps) {
	useCss(wrapperCss)
	return (
		<Layout>
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
