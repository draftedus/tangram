import { PinwheelInfo, h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default function MembersInvitePage(props: Props) {
	return (
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Invite Team Member'}</ui.H1>
				<ui.Form post={true}>
					<ui.TextField label="Email" name="email"></ui.TextField>
					<ui.CheckboxField label="Admin" name="isAdmin" />
					<ui.Button>{'Invite'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>
	)
}
