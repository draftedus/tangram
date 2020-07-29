import { css, cssClass, h, useCss } from './deps'
import { variables } from './theme'

type AvatarProps = {
	src?: string
}

let avatarClass = cssClass()
let avatarCss = css({
	[`.${avatarClass}`]: {
		backgroundColor: variables.colors.surface,
		borderRadius: '50%',
		height: '2rem',
		overflow: 'hidden',
		width: '2rem',
	},
	[`${avatarClass}:hover}`]: {
		filter: 'brightness(90%)',
	},
})

let imgClass = cssClass()
let imgCss = css({
	[`.${imgClass}`]: {
		border: 'none',
		height: '100%',
		outline: 'none',
		width: '100%',
	},
})

let placeholderClass = cssClass()
let placeholderCss = css({
	[`.${placeholderClass}`]: {
		backgroundColor: variables.colors.surface,
		height: '100%',
		width: '100%',
	},
})

export function Avatar(props: AvatarProps) {
	useCss(avatarCss)
	useCss(imgCss)
	useCss(placeholderCss)
	return (
		<div class={avatarClass}>
			{props.src ? (
				<div class={imgClass}>
					<img alt="avatar" src={props.src} />
				</div>
			) : (
				<div class={placeholderClass}>
					<DefaultAvatar />
				</div>
			)}
		</div>
	)
}

function DefaultAvatar() {
	return (
		<svg height="100%" viewBox="0 0 100 100" width="100%">
			<desc>avatar</desc>
			<circle cx="50" cy="50" fill={variables.colors.accent} r="50"></circle>
			<circle cx="50" cy="40" fill={variables.colors.surface} r="16"></circle>
			<circle cx="50" cy="96" fill={variables.colors.surface} r="36"></circle>
		</svg>
	)
}
