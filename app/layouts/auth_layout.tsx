import './auth_layout.css'
import * as ui from '@tangramhq/ui'
import { Document, Logo, LogoScheme } from '@tangramhq/www'
import { ComponentChildren, h } from 'preact'

type AuthLayoutProps = {
	children?: ComponentChildren
	clientJsSrc?: string
	cssSrcs?: string[]
	preloadJsSrcs?: string[]
}

export function AuthLayout(props: AuthLayoutProps) {
	return (
		<Document
			clientJsSrc={props.clientJsSrc}
			cssSrcs={props.cssSrcs}
			preloadJsSrcs={props.preloadJsSrcs}
		>
			<div class="auth-layout">
				<div class="auth-layout-logo-wrapper">
					<Logo colorScheme={LogoScheme.Multi} />
				</div>
				<div class="auth-layout-card-wrapper">
					<ui.Card>{props.children}</ui.Card>
				</div>
			</div>
		</Document>
	)
}
