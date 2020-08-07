import { Level } from './alert'
import { cx, h } from './deps'

type AlertProps = {
	alert: string
	level: Level
}

export function AlertIcon(props: AlertProps) {
	let levelClass
	switch (props.level) {
		case Level.Info:
			levelClass = 'alert-icon-level-info'
			break
		case Level.Warning:
			levelClass = 'alert-icon-level-warning'
			break
		case Level.Danger:
			levelClass = 'alert-icon-level-danger'
			break
	}

	let alertMessageClassCombined = cx('alert-icon-message', levelClass)
	let alertIconClassCombined = cx('alert-icon', levelClass)

	return (
		<div class="alert-icon-container">
			<div class={alertMessageClassCombined}>{props.alert}</div>
			<div class={alertIconClassCombined}>{'!'}</div>
		</div>
	)
}
