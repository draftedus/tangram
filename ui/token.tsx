import { Children, h } from './deps'

export type TokenProps = {
	children?: Children
	color?: string
	inline?: boolean
}

export function Token(props: TokenProps) {
	let style = props.color && {
		backgroundColor: props.color,
	}
	return (
		<div class="token" style={style}>
			{props.children}
		</div>
	)
}
