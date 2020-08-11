import { Logo, LogoScheme } from '../../www/components/logo'
import { Children, PinwheelInfo, h, ui } from 'deps'

type AuthLayoutProps = {
	children?: Children
	info: PinwheelInfo
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
		<div class="auth-layout">
			<div style={logoStyle}>
				<Logo color="var(--text-color)" colorScheme={LogoScheme.Multi} />
			</div>
			<div style={cardStyle}>
				<ui.Card>{props.children}</ui.Card>
			</div>
		</div>
	)
}
