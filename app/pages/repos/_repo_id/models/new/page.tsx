import { h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export default function ModelCreatePage() {
	return (
		<AppLayout>
			<ui.S1>
				<ui.H1>{'Upload Model'}</ui.H1>
				<ui.Form encType="multipart/form-data" post={true}>
					<ui.TextField label="Title" name="title" required={true} />
					<ui.FileField label="File" name="file" required={true} />
					<ui.Button type="submit">{'Upload'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>
	)
}
