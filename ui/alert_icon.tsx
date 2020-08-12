import { Level } from './alert'
import { cx, h } from './deps'

type AlertProps = {
	alert: string
	children: string
	level: Level
}

export function AlertIcon(props: AlertProps) {
	let levelClass: string
	switch (props.level) {
		case Level.Info:
			levelClass = 'alert-icon-level-info'
			break
		case Level.Success:
			levelClass = 'alert-icon-level-success'
			break
		case Level.Warning:
			levelClass = 'alert-icon-level-warning'
			break
		case Level.Danger:
			levelClass = 'alert-icon-level-danger'
			break
	}
	let alertMessageClass = cx('alert-icon-message', levelClass)
	let alertIconClass = cx('alert-icon', levelClass)
	return (
		<div class="alert-icon-container">
			<div class={alertMessageClass}>{props.alert}</div>
			<div class={alertIconClass}>{props.children}</div>
		</div>
	)
}
