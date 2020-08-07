import { Fragment, h } from './deps'

type ImageProps = {
	alt: string
	src: string
}

export function Img(props: ImageProps) {
	return (
		<Fragment>
			<details class="image-details">
				<summary class="image-details-summary">
					<img alt={props.alt} class="image-img" src={'/' + props.src} />
				</summary>
				<div class="image-viewer">
					<img alt={props.alt} class="image-viewer-img" src={'/' + props.src} />
				</div>
			</details>
		</Fragment>
	)
}
