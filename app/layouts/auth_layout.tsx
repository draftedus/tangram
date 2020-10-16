import './auth_layout.css'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { Document, Logo, LogoScheme } from '@tangramhq/www'
import { ComponentChildren, h } from 'preact'

type AuthLayoutProps = {
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function AuthLayout(props: AuthLayoutProps) {
	return (
		<Document pinwheelInfo={props.pinwheelInfo}>
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
