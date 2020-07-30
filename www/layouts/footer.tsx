import { Logo, LogoScheme } from '../components/logo'
import { css, cssClass, h, ui, useCss } from '../deps'

let gridClass = cssClass()
let gridCss = css({
	[`.${gridClass}`]: {
		alignItems: 'center',
		boxSizing: 'border-box',
		display: 'grid',
		grid: 'auto auto auto / auto',
		gridGap: '1rem',
		justifyItems: 'center',
		margin: '0 auto',
		maxWidth: ui.variables.width.max,
		padding: '2rem 1rem',
	},
})

let linksClass = cssClass()
let linksCss = css({
	[`.${linksClass}`]: {
		display: 'grid',
		grid: 'auto / auto auto auto',
		gridGap: '1rem',
	},
})

let copyrightClass = cssClass()
let copyrightCss = css({
	[`.${copyrightClass}`]: { color: 'gray', fontSize: '0.9rem', margin: '0' },
})

let logoClass = cssClass()
let logoCss = css({
	[`.${logoClass}`]: { height: '4rem', width: '4rem' },
})

export function Footer() {
	useCss(gridCss, linksCss, copyrightCss, logoCss)
	return (
		<div class={gridClass}>
			<nav class={linksClass}>
				<ui.Link href="/">Home</ui.Link>
				<ui.Link href="/pricing">Pricing</ui.Link>
				<ui.Link href="/docs">Docs</ui.Link>
			</nav>
			<Logo class={logoClass} colorScheme={LogoScheme.Multi} />
			<p class={copyrightClass}>Tangram © 2020</p>
		</div>
	)
}
