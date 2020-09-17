import './window.css'
import { ComponentChildren, h } from 'preact'

export enum WindowShade {
	Code,
	Default,
}

type WindowProps = {
	children?: ComponentChildren
}

export function Window(props: WindowProps) {
	return (
		<div class="window-wrapper">
			<div class="window-topbar">
				<div
					class="window-topbar-button"
					style={{ backgroundColor: 'var(--red)' }}
				/>
				<div
					class="window-topbar-button"
					style={{ backgroundColor: 'var(--yellow)' }}
				/>
				<div
					class="window-topbar-button"
					style={{ backgroundColor: 'var(--green)' }}
				/>
			</div>
			<div class="window-body">{props.children}</div>
		</div>
	)
}
