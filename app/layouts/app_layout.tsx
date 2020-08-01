import { TopbarLayout } from './topbar_layout'
import { Children, css, h, ui, useCss } from 'deps'

type AppLayoutProps = { children?: Children }

let appLayoutCss = css({
	[`.app-layout`]: {
		boxSizing: 'border-box',
		margin: '0 auto',
		maxWidth: ui.variables.width.max,
		padding: '2rem 1rem',
		width: '100%',
	},
})

export function AppLayout(props: AppLayoutProps) {
	useCss(appLayoutCss)
	return (
		<TopbarLayout>
			<div class="app-layout">{props.children}</div>
		</TopbarLayout>
	)
}
