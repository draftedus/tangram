import { h } from './deps'
import { variables } from './theme'

type AvatarProps = {
	src?: string
}

export function Avatar(props: AvatarProps) {
	return (
		<div class="avatar">
			{props.src ? (
				<div class="avatar-img">
					<img alt="avatar" src={props.src} />
				</div>
			) : (
				<div class="avatar-placeholder">
					<DefaultAvatar />
				</div>
			)}
		</div>
	)
}

function DefaultAvatar() {
	return (
		<svg height="100%" viewBox="0 0 100 100" width="100%">
			<desc>{'avatar'}</desc>
			<circle cx="50" cy="50" fill={variables.colors.accent} r="50"></circle>
			<circle cx="50" cy="40" fill={variables.colors.surface} r="16"></circle>
			<circle cx="50" cy="96" fill={variables.colors.surface} r="36"></circle>
		</svg>
	)
}
