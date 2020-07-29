import { h, ui } from '../deps'

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
		trapezoid = ui.colors.pink
		square = ui.colors.yellow
		mediumTriangle = ui.colors.teal
		smallTriangle1 = ui.colors.purple
		smallTriangle2 = ui.colors.indigo
		largeTriangle1 = ui.colors.blue
		largeTriangle2 = ui.colors.green
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
		<svg class={props.class} height="100%" viewBox="0 0 200 200" width="100%">
			<desc>tangram</desc>
			<polygon
				fill={trapezoid}
				points="4 9.657 4 98.343 46 140.343 46 51.657"
			/>
			<polygon
				fill={square}
				points="100 105.657 55.657 150 100 194.343 144.343 150"
			/>
			<polygon fill={mediumTriangle} points="4 109.657 4 196 90.343 196" />
			<polygon fill={smallTriangle1} points="54 59.657 54 140.343 94.343 100" />
			<polygon
				fill={smallTriangle2}
				points="150 155.657 109.657 196 190.343 196"
			/>
			<polygon fill={largeTriangle1} points="190.343 4 9.657 4 100 94.343" />
			<polygon
				fill={largeTriangle2}
				points="196 9.657 105.657 100 196 190.343"
			/>
		</svg>
	)
}
