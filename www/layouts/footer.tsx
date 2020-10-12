import './footer.css'
import * as ui from '@tangramhq/ui'
import { Logo, LogoScheme } from 'common/logo'
import { h } from 'preact'

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
