import { Level } from './alert'
import { cx } from '@tangramhq/pinwheel'
import { ComponentChildren, h } from 'preact'

type CalloutProps = {
	children?: ComponentChildren
	level: Level
	title?: string
}

export function Callout(props: CalloutProps) {
	let levelClass
	switch (props.level) {
		case Level.Danger:
			levelClass = 'callout-wrapper-danger'
			break
		case Level.Info:
			levelClass = 'callout-wrapper-info'
			break
		case Level.Warning:
			levelClass = 'callout-wrapper-warning'
			break
	}

	return (
		<div class={cx('callout-wrapper', levelClass)}>
			{props.title && <div class="callout-title">{props.title}</div>}
			<div class="callout-inner">{props.children}</div>
		</div>
	)
}
