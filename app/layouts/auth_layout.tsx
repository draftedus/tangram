import { Logo, LogoScheme } from '../../www/components/logo'
import { Children, css, h, ui, useCss } from 'deps'

type AuthLayoutProps = { children?: Children }

let authLayoutCss = css({
	[`.auth-layout`]: {
		display: 'grid',
		gridRowGap: '1rem',
		justifyContent: 'center',
		justifyItems: 'center',
		paddingTop: '1rem',
	},
})

export function AuthLayout(props: AuthLayoutProps) {
	useCss(authLayoutCss)

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
