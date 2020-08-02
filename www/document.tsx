import { Body, DocumentProps, Head, h } from './deps'

let globalCss = `body { background-color: var(--background); color: var(--text); font-family: JetBrains Mono, monospace, sans-serif; margin: 0; line-height: 1; } @font-face { font-family: "JetBrains Mono"; src: url("/jetbrainsmono.woff2") format("woff2"); font-display: swap; } a, button { outline: none } :root { --background: #FFFFFF; --border: #F2F2F2; --header: #EEEEEE; --hover: #CCCCCC; --surface: #FAFAFA; --muted-text: #666666; --text: #222222; --accent: #0A84FF; --fun-text: #ffffff } @media (prefers-color-scheme: dark) { :root { --background: #111111; --border: #212121; --header: #232323; --hover: #333333; --surface: #181818; --muted-text: #8B8B8B; --text: #E8E8E8; --accent: #0A84FF; --fun-text: #111111 } }`

// :root {
// 	scrollbar-width: 4px;
// 	scrollbar-color: var(--surface) var(--header);
// }

// ::-webkit-scrollbar {
// 	width: 4px;
// }

// ::-webkit-scrollbar-thumb {
// 	border-radius: 2px;
// 	background-color: green;
// }

// ::-webkit-scrollbar-track {
// 	background-color: var(--header);
// }

// let driftJs = `"use strict";!function(){var t=window.driftt=window.drift=window.driftt||[];if(!t.init){if(t.invoked)return void(window.console&&console.error&&console.error("Drift snippet included twice."));t.invoked=!0,t.methods=["identify","config","track","reset","debug","show","ping","page","hide","off","on"],t.factory=function(e){return function(){var i=Array.prototype.slice.call(arguments);return i.unshift(e),t.push(i),t}},t.methods.forEach(function(e){t[e]=t.factory(e)}),t.load=function(t){var e=3e5*Math.ceil(new Date/3e5),i=document.createElement("script");i.type="text/javascript",i.async=!0,i.crossorigin="anonymous",i.src="https://js.driftt.com/include/"+e+"/"+t+".js";var n=document.getElementsByTagName("script")[0];n.parentNode.insertBefore(i,n)}}}(),drift.SNIPPET_VERSION="0.3.1",drift.load("iaed56cgc2dc");`

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
				<style dangerouslySetInnerHTML={{ __html: globalCss }} />
				<title>Tangram</title>
				<meta content="Tangram" name="description" />
			</head>
			<body>
				<Body {...props} />
				{/* {!props.dev && (
					<script dangerouslySetInnerHTML={{ __html: driftJs }} />
				)} */}
			</body>
		</html>
	)
}
