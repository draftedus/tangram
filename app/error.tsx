import { PinwheelInfo, h, ui } from './deps'
import { AppLayout } from './layouts/app_layout'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default function ErrorPage(props: Props) {
	return (
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Error'}</ui.H1>
				<ui.P>
					{'An unexpected error occurred. Please try again or '}
					<ui.Link href="mailto:help@tangramhq.com">
						{'contact support'}
					</ui.Link>
					{'.'}
				</ui.P>
			</ui.S1>
		</AppLayout>
	)
}
