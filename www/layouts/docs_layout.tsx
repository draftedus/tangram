import { Children, PinwheelInfo, h, ui } from '../deps'
import { Layout } from './layout'

type DocsLayoutProps = {
	children?: Children
	pinwheelInfo: PinwheelInfo
	selectedPage: DocsPage
}

export function DocsLayout(props: DocsLayoutProps) {
	return (
		<Layout pinwheelInfo={props.pinwheelInfo}>
			<div class="docs-layout-wrapper">
				<PageNav selectedPage={props.selectedPage} />
				<div>{props.children}</div>
			</div>
		</Layout>
	)
}

export enum DocsPage {
	Home,
	Install,
	Train,
	Predict,
	Report,
	Tune,
	Monitor,
}

type PageNavProps = {
	selectedPage: DocsPage
}

function PageNav(props: PageNavProps) {
	return (
		<ui.NestedNav>
			<ui.NestedNavItem
				href="/docs/"
				selected={props.selectedPage === DocsPage.Home}
			>
				{'Home'}
			</ui.NestedNavItem>
			<ui.NestedNavSection>
				<ui.NestedNavSectionTitle>{'Install'}</ui.NestedNavSectionTitle>
				<ui.NestedNavItem
					href="/docs/install"
					selected={props.selectedPage === DocsPage.Install}
				>
					{'Install'}
				</ui.NestedNavItem>
			</ui.NestedNavSection>
			<ui.NestedNavSection>
				<ui.NestedNavSectionTitle>{'Getting Started'}</ui.NestedNavSectionTitle>
				<ui.NestedNavItem
					href="/docs/getting-started/train"
					selected={props.selectedPage === DocsPage.Train}
				>
					{'Train'}
				</ui.NestedNavItem>
				<ui.NestedNavItem
					href="/docs/getting-started/predict"
					selected={props.selectedPage === DocsPage.Predict}
				>
					{'Predict'}
				</ui.NestedNavItem>
				<ui.NestedNavItem
					href="/docs/getting-started/report"
					selected={props.selectedPage === DocsPage.Report}
				>
					{'Report'}
				</ui.NestedNavItem>
				<ui.NestedNavItem
					href="/docs/getting-started/tune"
					selected={props.selectedPage === DocsPage.Tune}
				>
					{'Tune'}
				</ui.NestedNavItem>
				<ui.NestedNavItem
					href="/docs/getting-started/monitor"
					selected={props.selectedPage === DocsPage.Monitor}
				>
					{'Monitor'}
				</ui.NestedNavItem>
			</ui.NestedNavSection>
		</ui.NestedNav>
	)
}
