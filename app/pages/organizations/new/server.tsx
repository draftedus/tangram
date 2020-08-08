import { h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export default function OrganizationCreatePage() {
	return (
		<AppLayout>
			<ui.S1>
				<ui.H1>{'Create New Organization'}</ui.H1>
				<ui.Form post={true}>
					<ui.TextField label="Name" name="name" required={true}></ui.TextField>
					<ui.Button>{'Create'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>
	)
}