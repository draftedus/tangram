import { h, ui } from './deps'
import { PageLayout } from './layouts/page_layout'

export default function ErrorPage() {
	return (
		<PageLayout>
			<ui.S1>
				<ui.H1>Error</ui.H1>
				<ui.P>
					An unexpected error occurred. Please try again or{' '}
					<ui.Link href="mailto:help@tangramhq.com">contact support</ui.Link>.
				</ui.P>
			</ui.S1>
		</PageLayout>
	)
}
