import { Logo, LogoScheme } from '../../www/components/logo'
import { Children, h, ui } from 'deps'

type AuthLayoutProps = { children?: Children }

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
				<Logo color={ui.variables.colors.text} colorScheme={LogoScheme.Multi} />
			</div>
			<div style={cardStyle}>
				<ui.Card>{props.children}</ui.Card>
			</div>
		</div>
	)
}
