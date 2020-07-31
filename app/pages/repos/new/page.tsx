import { h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export type Props = {
	organizations: Array<{ id: string; name: string }>
}

export default function ModelCreatePage(props: Props) {
	return (
		<AppLayout>
			<ui.S1>
				<ui.H1>Upload Model</ui.H1>
				<ui.Form encType="multipart/form-data" post>
					<ui.TextField label="Title" name="title" />
					<ui.SelectField label="Owner" name="organization_id">
						{props.organizations.map(({ id, name }) => (
							<option key={id} value={id}>
								{name}
							</option>
						))}
					</ui.SelectField>
					<ui.FileField label="File" name="file" />
					<ui.Button type="submit">Upload</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>
	)
}
