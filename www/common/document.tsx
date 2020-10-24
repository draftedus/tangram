import { ComponentChildren, h } from 'preact'

export type DocumentProps = {
	children?: ComponentChildren
	clientJsSrc?: string
	cssSrcs?: string[]
	preloadJsSrcs?: string[]
}

export function Document(props: DocumentProps) {
	return (
		<html lang="en">
			<head>
				<meta charSet="utf-8" />
				<meta content="width=device-width, initial-scale=1" name="viewport" />
				{props.preloadJsSrcs?.map(modulePath => (
					<link href={modulePath} key={modulePath} rel="modulepreload" />
				))}
				<link href="/favicon.png" rel="icon" type="image/png" />
				<link
					as="font"
					crossOrigin="true"
					href="/jetbrainsmono.woff2"
					rel="preload"
					type="font/woff2"
				/>
				<link href="/ui.css" rel="stylesheet" />
				<link href="/charts.css" rel="stylesheet" />
				<link href="/www.css" rel="stylesheet" />
				<link href="/app.css" rel="stylesheet" />
				<title>{'Tangram'}</title>
				<meta
					content="All-In-One Machine Learning Toolkit for Developers"
					name="description"
				/>
			</head>
			<body>
				{props.children}
				{props.clientJsSrc && <script src={props.clientJsSrc} type="module" />}
			</body>
		</html>
	)
}
