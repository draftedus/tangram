import { PinwheelInfo } from './types'
import { ComponentChildren, Fragment, h } from 'preact'

export type HeadProps = {
	pinwheelInfo: PinwheelInfo
}

export function Head(props: HeadProps) {
	return (
		<Fragment>
			<meta charSet="utf-8" />
			<meta content="width=device-width, initial-scale=1" name="viewport" />
			{props.pinwheelInfo.preloadJsSrcs?.map(modulePath => (
				<link href={modulePath} key={modulePath} rel="modulepreload" />
			))}
		</Fragment>
	)
}

export type BodyProps = {
	children: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function Body(props: BodyProps) {
	return (
		<Fragment>
			{props.children}
			{props.pinwheelInfo.clientJsSrc && (
				<script src={props.pinwheelInfo.clientJsSrc} type="module" />
			)}
		</Fragment>
	)
}

export type DocumentProps = {
	children: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export default function Document(props: DocumentProps) {
	return (
		<Fragment>
			<html>
				<head>
					<Head pinwheelInfo={props.pinwheelInfo} />
				</head>
				<body>
					<Body children={props.children} pinwheelInfo={props.pinwheelInfo} />
				</body>
			</html>
		</Fragment>
	)
}
