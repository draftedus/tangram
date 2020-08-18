import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export type Props = {
	flash: string | null
	owners: Array<{ id: string; title: string }>
	pinwheelInfo: PinwheelInfo
}

export default function RepoCreatePage(props: Props) {
	return renderPage(
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Create New Repo'}</ui.H1>
				<ui.Form encType="multipart/form-data" post={true}>
					{props.flash && (
						<ui.Alert level={ui.Level.Danger}>{props.flash}</ui.Alert>
					)}
					<ui.TextField label="Title" name="title" required={true} />
					<ui.SelectField label="Owner" name="owner_id" required={true}>
						{props.owners.map(({ id, title }) => (
							<option key={id} value={id}>
								{title}
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
