import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { AuthLayout } from 'layouts/auth_layout'

export type LoginProps = {
	code?: boolean
	email?: string
	flash?: string
	pinwheelInfo: PinwheelInfo
}

export default function Login(props: LoginProps) {
	return renderPage(
		<AuthLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.Form post={true}>
				{props.flash && (
					<ui.Alert level={ui.Level.Danger}>{props.flash}</ui.Alert>
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
