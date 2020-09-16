import type { PinwheelInfo } from './mod'
import { ComponentChildren, Fragment, h } from 'preact'

export type HeadProps = {
	pinwheelInfo: PinwheelInfo
}

export function Head(props: HeadProps) {
	return (
		<>
			<meta charSet="utf-8" />
			<meta content="width=device-width, initial-scale=1" name="viewport" />
			{props.pinwheelInfo.preloadJsSrcs?.map(modulePath => (
				<link href={modulePath} key={modulePath} rel="modulepreload" />
			))}
		</>
	)
}

export type BodyProps = {
	children: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function Body(props: BodyProps) {
	return (
		<>
			{props.children}
			{props.pinwheelInfo.clientJsSrc && (
				<script src={props.pinwheelInfo.clientJsSrc} type="module" />
			)}
		</>
	)
}

export type DocumentProps = {
	children: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function Document(props: DocumentProps) {
	return (
		<>
			<html>
				<head>
					<Head pinwheelInfo={props.pinwheelInfo} />
				</head>
				<body>
					<Body children={props.children} pinwheelInfo={props.pinwheelInfo} />
				</body>
			</html>
		</>
	)
}
