import { Fragment, css, h, useCss } from './deps'

type ImageProps = {
	alt: string
	src: string
}

let detailsCss = css({
	['.image-details[open] > summary::before']: {
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

let summaryCss = css({
	[`.image-details-summary`]: {
		display: 'grid',
	},
	[`.image-details-summary:-webkit-details-marker`]: {
		display: 'none',
	},
})

let viewerCss = css({
	[`.image-viewer`]: {
		backgroundColor: 'rgba(0, 0, 0, 0.8)',
		bottom: '0',
		left: '0',
		padding: '1rem',
		position: 'fixed',
		right: '0',
		top: '0',
	},
})

let viewerImgCss = css({
	[`.image-viewer-img`]: {
		height: '100%',
		objectFit: 'contain',
		width: '100%',
	},
})

let imgCss = css({
	[`.image-img`]: { borderRadius: '4px', cursor: 'zoom-in', width: '100%' },
})

export function Img(props: ImageProps) {
	useCss(detailsCss, summaryCss, viewerCss, viewerImgCss, imgCss)
	return (
		<Fragment>
			<details class="image-details">
				<summary class="image-details-summary">
					<img alt={props.alt} class="image-img" src={props.src} />
				</summary>
				<div class="image-viewer">
					<img alt={props.alt} class="image-viewer-img" src={props.src} />
				</div>
			</details>
		</Fragment>
	)
}
