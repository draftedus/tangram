import { Logo, LogoScheme } from '../components/logo'
import { h, ui } from '../deps'

export function Footer() {
	return (
		<div class="footer-grid">
			<nav class="footer-links-wrapper">
				<ui.Link href="/">{'Home'}</ui.Link>
				<ui.Link href="/pricing">{'Pricing'}</ui.Link>
				<ui.Link href="/docs">{'Docs'}</ui.Link>
			</nav>
			<Logo class="footer-logo" colorScheme={LogoScheme.Multi} />
			<p class="footer-copyright">{'Tangram © 2020'}</p>
		</div>
	)
}
