import { h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export default function MembersInvitePage() {
	return (
		<AppLayout>
			<ui.S1>
				<ui.H1>Invite Team Member</ui.H1>
				<ui.Form>
					<ui.TextField label="Email" name="email"></ui.TextField>
					<ui.Button>Invite</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>
	)
}
