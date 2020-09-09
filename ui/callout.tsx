import { Level } from './alert'
import { ComponentChildren, h } from 'preact'

type CalloutProps = {
	children?: ComponentChildren
	level: Level
	title?: string
}

export function Callout(props: CalloutProps) {
	let wrapperClass
	switch (props.level) {
		case Level.Danger:
			wrapperClass = 'callout-level-danger-wrapper'
			break
		case Level.Info:
			wrapperClass = 'callout-level-info-wrapper'
			break
		case Level.Warning:
			wrapperClass = 'callout-level-warning-wrapper'
			break
	}

	return (
		<div class={wrapperClass}>
			<div class="callout-wrapper">
				{props.title && <div class="callout-title">{props.title}</div>}
				<div class="callout-inner">{props.children}</div>
			</div>
		</div>
	)
}
