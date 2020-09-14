import { PageLayout } from './layouts/page_layout'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default function NotFoundPage(props: Props) {
	return (
		<PageLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Not Found'}</ui.H1>
				<ui.P>{'We were unable to find the page you requested.'}</ui.P>
			</ui.S1>
		</PageLayout>
	)
}
