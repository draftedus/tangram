import { h, ui } from 'deps'
import { AuthLayout } from 'layouts/auth_layout'

export type LoginProps = {
	code?: boolean
	email?: string
	error?: string
}

export default function Login(props: LoginProps) {
	return (
		<AuthLayout>
			<ui.Form directive={props.code ? 'code' : 'email'} post>
				{props.error && (
					<ui.Alert level={ui.Level.Danger}>{props.error}</ui.Alert>
				)}
				<ui.TextField
					autocomplete="username"
					disabled={!props.email}
					name="email"
					placeholder="Email"
					value={props.email}
				/>
				{props.code && <ui.TextField name="code" placeholder="Code" />}
				<ui.Button type="submit">Login</ui.Button>
				{props.code && (
					<div style={{ lineHeight: '1.5', textAlign: 'center' }}>
						We emailed you a code. Copy and paste it above to log in.
					</div>
				)}
			</ui.Form>
		</AuthLayout>
	)
}
