import { h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export default function ModelCreatePage() {
	return (
		<AppLayout>
			<ui.S1>
				<ui.H1>Upload Model</ui.H1>
				<form method="post">
					<input name="title" type="text" />
					<input name="file" type="file" />
					<ui.Button type="submit">Upload</ui.Button>
				</form>
			</ui.S1>
		</AppLayout>
	)
}
