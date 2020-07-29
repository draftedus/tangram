import { h, ui } from './deps'
import { AppLayout } from './layouts/app_layout'

export default function NotFoundPage() {
	return (
		<AppLayout>
			<ui.S1>
				<ui.H1>Not Found</ui.H1>
				<ui.P>We were unable to find the page you requested.</ui.P>
			</ui.S1>
		</AppLayout>
	)
}
