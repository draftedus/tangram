import { Children, h } from './deps'

export enum Level {
	Info,
	Success,
	Warning,
	Danger,
}

export type AlertProps = {
	children?: Children
	level: Level
	title?: string
}

export function Alert(props: AlertProps) {
	let levelClass: string
	switch (props.level) {
		case Level.Info:
			levelClass = 'alert-level-info'
			break
		case Level.Success:
			levelClass = 'alert-level-success'
			break
		case Level.Warning:
			levelClass = 'alert-level-warning'
			break
		case Level.Danger:
			levelClass = 'alert-level-danger'
			break
	}
	return (
		<div class={`alert-wrapper ${levelClass}`}>
			{props.title && <div class="alert-title">{props.title}</div>}
			{props.children}
		</div>
	)
}
