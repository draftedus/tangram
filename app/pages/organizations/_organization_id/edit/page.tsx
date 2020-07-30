import { h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export default function OrganizationEditPage() {
	return (
		<AppLayout>
			<ui.S1>
				<ui.H1>Edit Organization</ui.H1>
				<ui.Form post>
					<ui.TextField label="Organization Name" name="name" />
					<ui.Button type="submit">Submit</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>
	)
}
