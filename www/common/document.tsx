import { PageInfo } from '@tangramhq/pinwheel'
import { ComponentChildren, h } from 'preact'

export type DocumentProps = {
	children?: ComponentChildren
	pageInfo: PageInfo
}

export function Document(props: DocumentProps) {
	return (
		<html lang="en">
			<head>
				<meta charSet="utf-8" />
				<meta content="width=device-width, initial-scale=1" name="viewport" />
				{props.pageInfo.preloadJsSrcs?.map(modulePath => (
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
				<script>
					{
						'document.cookie = `tangram-timezone=${Intl.DateTimeFormat().resolvedOptions().timeZone};max-age=31536000`'
					}
				</script>
				{props.pageInfo.clientJsSrc && (
					<script src={props.pageInfo.clientJsSrc} type="module" />
				)}
			</body>
		</html>
	)
}
