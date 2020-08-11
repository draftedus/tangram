import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export type Props = {
	info: PinwheelInfo
	organizations: Array<{ id: string; name: string }>
}

export default function RepoCreatePage(props: Props) {
	return renderPage(
		<AppLayout info={props.info}>
			<ui.S1>
				<ui.H1>{'Upload Model'}</ui.H1>
				<ui.Form encType="multipart/form-data" post={true}>
					<ui.TextField label="Title" name="title" required={true} />
					<ui.SelectField label="Owner" name="organization_id" required={true}>
						{props.organizations.map(({ id, name }) => (
							<option key={id} value={id}>
								{name}
							</option>
						))}
					</ui.SelectField>
					<ui.FileField label="File" name="file" required={true} />
					<ui.Button type="submit">{'Upload'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>,
	)
}
