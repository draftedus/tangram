import { Body, Head, PinwheelInfo } from '@tangramhq/pinwheel'
import { ComponentChildren, h } from 'preact'

export type DocumentProps = {
	children?: ComponentChildren
	pinwheelInfo: PinwheelInfo
}

export function Document(props: DocumentProps) {
	return (
		<html lang="en">
			<head>
				<Head pinwheelInfo={props.pinwheelInfo} />
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
				<Body pinwheelInfo={props.pinwheelInfo}>{props.children}</Body>
			</body>
		</html>
	)
}
