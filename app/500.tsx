import { AppLayout } from './layouts/app_layout'
import { PinwheelInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default function ErrorPage(props: Props) {
	return (
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Error'}</ui.H1>
				<ui.P>
					{
						'An unexpected error occurred. Please try again or send an email to '
					}
					<ui.Link href="mailto:help@tangramhq.com">
						{'help@tangramhq.com'}
					</ui.Link>
					{'.'}
				</ui.P>
			</ui.S1>
		</AppLayout>
	)
}