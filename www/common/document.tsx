import { PageInfo } from '@tangramhq/pinwheel'
import '@tangramhq/ui/global.css'
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
				<link href="/favicon.png" rel="icon" type="image/png" />
				{props.pageInfo.preloadJsSrcs?.map(modulePath => (
					<link href={modulePath} key={modulePath} rel="modulepreload" />
				))}
				{props.pageInfo.cssSrcs?.map(cssSrc => (
					<link href={cssSrc} key={cssSrc} rel="stylesheet" />
				))}
				<title>{'Tangram'}</title>
				<meta
					content="All-In-One Machine Learning Toolkit for Programmers"
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
