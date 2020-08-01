import { Logo, LogoScheme } from '../components/logo'
import { css, h, ui, useCss } from '../deps'

let gridCss = css({
	[`.footer-grid`]: {
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

let linksCss = css({
	[`.footer-links-wrapper`]: {
		display: 'grid',
		grid: 'auto / auto auto auto',
		gridGap: '1rem',
	},
})

let copyrightCss = css({
	[`.footer-copyright`]: { color: 'gray', fontSize: '0.9rem', margin: '0' },
})

let logoCss = css({
	[`.footer-logo`]: { height: '4rem', width: '4rem' },
})

export function Footer() {
	useCss(gridCss, linksCss, copyrightCss, logoCss)
	return (
		<div class="footer-grid">
			<nav class="footer-links-wrapper">
				<ui.Link href="/">Home</ui.Link>
				<ui.Link href="/pricing">Pricing</ui.Link>
				<ui.Link href="/docs">Docs</ui.Link>
			</nav>
			<Logo class="footer-logo" colorScheme={LogoScheme.Multi} />
			<p class="footer-copyright">Tangram © 2020</p>
		</div>
	)
}
