import { h } from 'preact'

export enum LogoScheme {
	Multi,
	Solid,
}

type LogoProps = {
	class?: string
	color?: string
	colorScheme: LogoScheme
}

export function Logo(props: LogoProps) {
	let trapezoid
	let square
	let mediumTriangle
	let smallTriangle1
	let smallTriangle2
	let largeTriangle1
	let largeTriangle2
	if (props.colorScheme === LogoScheme.Multi) {
		trapezoid = 'var(--pink)'
		square = 'var(--yellow)'
		mediumTriangle = 'var(--teal)'
		smallTriangle1 = 'var(--purple)'
		smallTriangle2 = 'var(--indigo)'
		largeTriangle1 = 'var(--blue)'
		largeTriangle2 = 'var(--green)'
	} else {
		trapezoid = props.color
		square = props.color
		mediumTriangle = props.color
		smallTriangle1 = props.color
		smallTriangle2 = props.color
		largeTriangle1 = props.color
		largeTriangle2 = props.color
	}
	return (
		<svg
			class={props.class}
			fill="none"
			height="100%"
			viewBox="0 0 200 200"
			width="100%"
			xmlns="http://www.w3.org/2000/svg"
		>
			<desc>{'tangram'}</desc>
			<path d="M0 0L50 50V150L0 100V0Z" fill={trapezoid} />
			<path d="M100 100L150 150L100 200L50 150L100 100Z" fill={square} />
			<path d="M0 100L100 200H0V100Z" fill={mediumTriangle} />
			<path d="M150 150L200 200H100L150 150Z" fill={smallTriangle2} />
			<path d="M50 50L100 100L50 150V50Z" fill={smallTriangle1} />
			<path d="M200 0V200L100 100L200 0Z" fill={largeTriangle2} />
			<path d="M200 0L100 100L0 0H200Z" fill={largeTriangle1} />
		</svg>
	)
}
