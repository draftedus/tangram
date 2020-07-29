import { Children, css, cssClass, cx, h, useCss } from './deps'

type Props = { children?: Children }

let s1Class = cssClass()
let s1Css = css({
	[`.${s1Class}`]: {
		alignContent: 'start',
		display: 'grid',
		gridRowGap: '2rem',
	},
})

export function S1(props: Props) {
	useCss(s1Css)
	return <div class={s1Class}>{props.children}</div>
}

let s2Class = cssClass()
let s2Css = css({
	[`.${s2Class}`]: {
		display: 'grid',
		gridRowGap: '1rem',
	},
})

export function S2(props: Props) {
	useCss(s2Css)
	return <div class={s2Class}>{props.children}</div>
}

let spaceBetweenClass = cssClass()
let spaceBetweenCss = css({
	[`.${spaceBetweenClass}`]: {
		alignItems: 'center',
		display: 'grid',
		gridAutoFlow: 'column',
		justifyContent: 'space-between',
	},
})

export function SpaceBetween(props: Props) {
	useCss(spaceBetweenCss)
	return <div class={spaceBetweenClass}>{props.children}</div>
}

type HProps = {
	center?: boolean
	children?: Children
}

let h1Class = cssClass()
let h1Css = css({
	[`.${h1Class}`]: {
		fontSize: '2.5rem',
		fontWeight: 'normal',
		margin: '0',
	},
})

let centeredClass = cssClass()
let centeredCss = css({
	[`.${centeredClass}`]: { textAlign: 'center' },
})

export function H1(props: HProps) {
	useCss(h1Css, centeredCss)
	return (
		<h1 class={cx(h1Class, props.center && centeredClass)}>{props.children}</h1>
	)
}

let h2Class = cssClass()
let h2Css = css({
	[`.${h2Class}`]: {
		fontSize: '1.5rem',
		fontWeight: 'normal',
		margin: '0',
	},
})

export function H2(props: HProps) {
	useCss(h2Css, centeredCss)
	return (
		<h2 class={cx(h2Class, props.center && centeredClass)}>{props.children}</h2>
	)
}

let pClass = cssClass()
let pCss = css({
	[`.${pClass}`]: { lineHeight: '1.5', margin: '0' },
})

export function P(props: Props) {
	useCss(pCss)
	return <p class={pClass}>{props.children}</p>
}

let listClass = cssClass()
let listCss = css({
	[`.${listClass}`]: {
		lineHeight: '1.5',
		listStylePosition: 'inside',
		margin: '0',
		padding: '0',
	},
})

export function List(props: Props) {
	useCss(listCss)
	return <ul class={listClass}>{props.children}</ul>
}

let orderedListClass = cssClass()
let orderedListCss = css({
	[`.${orderedListClass}`]: {
		lineHeight: '1.5',
		listStylePosition: 'inside',
		margin: '0',
		padding: '0',
	},
})

export function OrderedList(props: Props) {
	useCss(orderedListCss)
	return <ol class={orderedListClass}>{props.children}</ol>
}

export function ListItem(props: Props) {
	return <li>{props.children}</li>
}
