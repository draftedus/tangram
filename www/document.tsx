import { Body, DocumentProps, Head, h } from './deps'

export * from '../pinwheel/lib/mod'

export default function Document(props: DocumentProps) {
	return (
		<html lang="en">
			<head>
				<Head {...props} />
				<link
					as="font"
					crossOrigin="true"
					href="/jetbrainsmono.woff2"
					rel="preload"
					type="font/woff2"
				/>
				<link href="/tangram.css" rel="stylesheet" />
				<title>{'Tangram'}</title>
				<meta content="Machine Learning for Developers" name="description" />
			</head>
			<body>
				<Body {...props} />
			</body>
		</html>
	)
}
