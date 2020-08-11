import { Children, Fragment, h } from './react'
import { PinwheelInfo } from './types'

export type HeadProps = {
	info: PinwheelInfo
}

export function Head(props: HeadProps) {
	return (
		<Fragment>
			<meta charSet="utf-8" />
			<meta content="width=device-width, initial-scale=1" name="viewport" />
			{props.info.preloadJsSrcs?.map(modulePath => (
				<link href={modulePath} key={modulePath} rel="modulepreload" />
			))}
		</Fragment>
	)
}

export type BodyProps = {
	children: Children
	info: PinwheelInfo
}

export function Body(props: BodyProps) {
	return (
		<Fragment>
			{props.children}
			{props.info.clientJsSrc && (
				<script src={props.info.clientJsSrc} type="module" />
			)}
		</Fragment>
	)
}

export type DocumentProps = {
	children: Children
	info: PinwheelInfo
}

export default function Document(props: DocumentProps) {
	return (
		<Fragment>
			<html>
				<head>
					<Head info={props.info} />
				</head>
				<body>
					<Body children={props.children} info={props.info} />
				</body>
			</html>
		</Fragment>
	)
}
