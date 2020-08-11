import { renderPage } from '../../../../../ui/deps'
import { PinwheelInfo, h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default function OrganizationEditPage(props: Props) {
	return renderPage(
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Edit Organization'}</ui.H1>
				<ui.Form post={true}>
					<ui.TextField label="Organization Name" name="name" />
					<ui.Button type="submit">{'Submit'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>,
	)
}
