import { Client, h, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

export default function ModelCreatePage() {
	return (
		<Client
			component={() => {
				return (
					<AppLayout>
						<ui.S1>
							<ui.H1>Upload Model</ui.H1>
							<ui.Form encType="multipart/form-data" post>
								<ui.TextField label="Title" name="title" />
								<ui.FileField label="File" name="file" />
								<ui.Button type="submit">Upload</ui.Button>
							</ui.Form>
						</ui.S1>
					</AppLayout>
				)
			}}
			id="model-create-page"
			props={{}}
		/>
	)
}
