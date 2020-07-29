import { Children, css, cssClass, h, useCss } from './deps'

type HeaderProps = { children?: Children }

let headerClass = cssClass()
let headerCss = css({
	[`.${headerClass}`]: {
		alignItems: 'center',
		display: 'grid',
		grid: 'auto / 1fr auto',
		gridColumnGap: '10px',
		justifyItems: 'stretch',
	},
})

export function Header(props: HeaderProps) {
	useCss(headerCss)
	return <div class={headerClass}>{props.children}</div>
}
