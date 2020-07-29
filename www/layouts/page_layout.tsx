import { Children, css, cssClass, h, ui, useCss } from '../deps'
import { Layout } from './layout'

type PageLayoutProps = {
	background?: boolean
	children?: Children
}

let wrapperClass = cssClass()
let wrapperCss = css({
	[`.${wrapperClass}`]: {
		boxSizing: 'border-box',
		margin: '0 auto',
		maxWidth: ui.variables.width.max,
		padding: '2rem 1rem',
		width: '100%',
	},
})

export function PageLayout(props: PageLayoutProps) {
	useCss(wrapperCss)
	return (
		<Layout background={props.background}>
			<div class={wrapperClass}>{props.children}</div>
		</Layout>
	)
}
