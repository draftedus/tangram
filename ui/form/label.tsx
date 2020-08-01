import { Children, css, h, useCss } from '../deps'
import { variables } from '../theme'

type Props = {
	children?: Children
	for?: string
}

let labelCss = css({
	[`.form-label`]: {
		color: variables.colors.text,
		display: 'grid',
		gridGap: '0.5rem',
		userSelect: 'none',
	},
})

export function Label(props: Props) {
	useCss(labelCss)
	return (
		<label class="form-label" htmlFor={props.for}>
			{props.children}
		</label>
	)
}
