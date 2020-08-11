import { PinwheelInfo, h, renderPage, ui } from 'deps'
import { AppLayout } from 'layouts/app_layout'

type Props = {
	pinwheelInfo: PinwheelInfo
}

export default function ModelCreatePage(props: Props) {
	return renderPage(
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.H1>{'Upload Model'}</ui.H1>
				<ui.Form encType="multipart/form-data" post={true}>
					<ui.TextField label="Title" name="title" required={true} />
					<ui.FileField label="File" name="file" required={true} />
					<ui.Button type="submit">{'Upload'}</ui.Button>
				</ui.Form>
			</ui.S1>
		</AppLayout>,
	)
}
