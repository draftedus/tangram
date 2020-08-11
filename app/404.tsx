import { PinwheelInfo, h, ui } from './deps'
import { AppLayout } from './layouts/app_layout'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default function NotFoundPage(props: Props) {
	return (
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Not Found'}</ui.H1>
				<ui.P>{'We were unable to find the page you requested.'}</ui.P>
			</ui.S1>
		</AppLayout>
	)
}
