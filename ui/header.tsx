import { Children, css, h, useCss } from './deps'

type HeaderProps = { children?: Children }

let headerCss = css({
	[`.header`]: {
		alignItems: 'center',
		display: 'grid',
		grid: 'auto / 1fr auto',
		gridColumnGap: '10px',
		justifyItems: 'stretch',
	},
})

export function Header(props: HeaderProps) {
	useCss(headerCss)
	return <div class="header">{props.children}</div>
}
