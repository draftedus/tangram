import { Body, DocumentProps, Head, h } from './deps'

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
				<style src="/global.css" />
				<title>{'Tangram'}</title>
				<meta content="Tangram" name="description" />
			</head>
			<body>
				<Body {...props} />
			</body>
		</html>
	)
}
