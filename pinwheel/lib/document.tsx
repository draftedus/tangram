import { Fragment, h } from './react'
import { DocumentProps } from './types'

export function Head(props: DocumentProps) {
	return (
		<Fragment>
			<meta charSet="utf-8" />
			<meta content="width=device-width, initial-scale=1" name="viewport" />
			{props.preloadJsPaths?.map(modulePath => (
				<link href={modulePath} key={modulePath} rel="modulepreload" />
			))}
		</Fragment>
	)
}

export function Body(props: DocumentProps) {
	return (
		<Fragment>
			{props.html && (
				<div dangerouslySetInnerHTML={{ __html: props.html }} id="root" />
			)}
			{props.clientJsPath && <script src={props.clientJsPath} type="module" />}
		</Fragment>
	)
}

export default function Document(props: DocumentProps) {
	return (
		<html>
			<head>
				<Head {...props} />
			</head>
			<body>
				<Body {...props} />
			</body>
		</html>
	)
}
