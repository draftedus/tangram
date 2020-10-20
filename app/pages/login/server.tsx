import { Props } from './props'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { AuthLayout } from 'layouts/auth_layout'
import { h } from 'preact'

export default function Login(props: Props) {
	return renderPage(
		<AuthLayout pinwheelInfo={props.pinwheelInfo}>
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
					<div class="login-code-message">
						{'We emailed you a code. Copy and paste it above to log in.'}
					</div>
				)}
			</ui.Form>
		</AuthLayout>,
	)
}
