import { Children, css, cx, h, useCss } from './deps'

type Props = { children?: Children }

let s1Css = css({
	[`.s1`]: {
		alignContent: 'start',
		display: 'grid',
		gridRowGap: '2rem',
	},
})

export function S1(props: Props) {
	useCss(s1Css)
	return <div class="s1">{props.children}</div>
}

let s2Css = css({
	[`.s2`]: {
		display: 'grid',
		gridRowGap: '1rem',
	},
})

export function S2(props: Props) {
	useCss(s2Css)
	return <div class="s2">{props.children}</div>
}

let spaceBetweenCss = css({
	[`.space-between`]: {
		alignItems: 'center',
		display: 'grid',
		gridAutoFlow: 'column',
		justifyContent: 'space-between',
	},
})

export function SpaceBetween(props: Props) {
	useCss(spaceBetweenCss)
	return <div class="space-between">{props.children}</div>
}

type HProps = {
	center?: boolean
	children?: Children
}

let h1Css = css({
	[`.h1`]: {
		fontSize: '2.5rem',
		fontWeight: 'normal',
		margin: '0',
	},
})

let centeredCss = css({
	[`.centered`]: { textAlign: 'center' },
})

export function H1(props: HProps) {
	useCss(h1Css, centeredCss)
	return <h1 class={cx('h1', props.center && 'centered')}>{props.children}</h1>
}

let h2Css = css({
	[`.h2`]: {
		fontSize: '1.5rem',
		fontWeight: 'normal',
		margin: '0',
	},
})

export function H2(props: HProps) {
	useCss(h2Css, centeredCss)
	return <h2 class={cx('h2', props.center && 'centered')}>{props.children}</h2>
}

let pCss = css({
	[`.p`]: { lineHeight: '1.5', margin: '0' },
})

export function P(props: Props) {
	useCss(pCss)
	return <p class="p">{props.children}</p>
}

let listCss = css({
	[`.list`]: {
		lineHeight: '1.5',
		listStylePosition: 'inside',
		margin: '0',
		padding: '0',
	},
})

export function List(props: Props) {
	useCss(listCss)
	return <ul class="list">{props.children}</ul>
}

let orderedListCss = css({
	[`.ordered-list`]: {
		lineHeight: '1.5',
		listStylePosition: 'inside',
		margin: '0',
		padding: '0',
	},
})

export function OrderedList(props: Props) {
	useCss(orderedListCss)
	return <ol class="ordered-list">{props.children}</ol>
}

export function ListItem(props: Props) {
	return <li>{props.children}</li>
}
