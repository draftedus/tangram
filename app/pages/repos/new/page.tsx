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
				<ui.Form>
					<ui.TextField label="Title" name="title" />
					<ui.SelectField
						label="Owner"
						name="org"
						options={['root', ...props.organizations.map(org => org.name)]}
					/>
					<ui.TextField label="File" name="file" type="file" />
					<ui.Button type="submit">Upload</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>
	)
}
