import { Fragment, css, cssClass, h, useCss } from './deps'

type ImageProps = {
	alt: string
	src: string
}

let detailsClass = cssClass()
let detailsCss = css({
	['.${detailsClass}[open] > summary::before']: {
		background: 'transparent',
		bottom: '0',
		content: '" "',
		cursor: 'zoom-out',
		display: 'block',
		left: '0',
		position: 'fixed',
		right: '0',
		top: '0',
		zIndex: 1,
	},
})

let summaryClass = cssClass()
let summaryCss = css({
	[`.${summaryClass}`]: {
		display: 'grid',
	},
	[`.${summaryClass}:-webkit-details-marker`]: {
		display: 'none',
	},
})

let viewerClass = cssClass()
let viewerCss = css({
	[`.${viewerClass}`]: {
		backgroundColor: 'rgba(0, 0, 0, 0.8)',
		bottom: '0',
		left: '0',
		padding: '1rem',
		position: 'fixed',
		right: '0',
		top: '0',
	},
})

let viewerImgClass = cssClass()
let viewerImgCss = css({
	[`.${viewerImgClass}`]: {
		height: '100%',
		objectFit: 'contain',
		width: '100%',
	},
})

let imgClass = cssClass()
let imgCss = css({
	[`.${imgClass}`]: { borderRadius: '4px', cursor: 'zoom-in', width: '100%' },
})

export function Img(props: ImageProps) {
	useCss(detailsCss, summaryCss, viewerCss, viewerImgCss, imgCss)
	return (
		<Fragment>
			<details class={detailsClass}>
				<summary class={summaryClass}>
					<img alt={props.alt} class={imgClass} src={props.src} />
				</summary>
				<div class={viewerClass}>
					<img alt={props.alt} class={viewerImgClass} src={props.src} />
				</div>
			</details>
		</Fragment>
	)
}
