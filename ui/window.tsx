import { Children, h } from './deps'
import { colors } from './theme'

export enum WindowShade {
	Code,
	Default,
}

type WindowProps = {
	children?: Children
}

export function Window(props: WindowProps) {
	return (
		<div class="window-wrapper">
			<div class="window-topbar">
				<div
					class="window-topbar-button"
					style={{ backgroundColor: colors.red }}
				/>
				<div
					class="window-topbar-button"
					style={{ backgroundColor: colors.yellow }}
				/>
				<div
					class="window-topbar-button"
					style={{ backgroundColor: colors.green }}
				/>
			</div>
			<div class="window-body">{props.children}</div>
		</div>
	)
}
