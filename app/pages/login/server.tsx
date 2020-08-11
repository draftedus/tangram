import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { AuthLayout } from 'layouts/auth_layout'

export type LoginProps = {
	code?: boolean
	email?: string
	error?: string
	info: PinwheelInfo
}

export default function Login(props: LoginProps) {
	return renderPage(
		<AuthLayout info={props.info}>
			<ui.Form post={true}>
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
				<ui.Button type="submit">{'Login'}</ui.Button>
				{props.code && (
					<div style={{ lineHeight: '1.5', textAlign: 'center' }}>
						{'We emailed you a code. Copy and paste it above to log in.'}
					</div>
				)}
			</ui.Form>
		</AuthLayout>,
	)
}
