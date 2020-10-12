import './auth_layout.css'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { Document } from '@tangramhq/www/common/document'
import { Logo, LogoScheme } from '@tangramhq/www/common/logo'
import { ComponentChildren, h } from 'preact'

type AuthLayoutProps = {
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function AuthLayout(props: AuthLayoutProps) {
	let logoStyle = {
		height: '5rem',
		width: '5rem',
	}
	let cardStyle = {
		width: '300px',
	}
	return (
		<Document pinwheelInfo={props.pinwheelInfo}>
			<div class="auth-layout">
				<div style={logoStyle}>
					<Logo color="var(--text-color)" colorScheme={LogoScheme.Multi} />
				</div>
				<div style={cardStyle}>
					<ui.Card>{props.children}</ui.Card>
				</div>
			</div>
		</Document>
	)
}
