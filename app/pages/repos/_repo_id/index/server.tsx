import { Props } from './props'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { AppLayout } from 'layouts/app_layout'
import { h } from 'preact'

export default function HomePage(props: Props) {
	return renderPage(
		<AppLayout pinwheelInfo={props.pinwheelInfo}>
			<ui.S1>
				<ui.SpaceBetween>
					<ui.H1>{'Models'}</ui.H1>
					<ui.Button href="./models/new">{'Upload New Version'}</ui.Button>
				</ui.SpaceBetween>
				{props.models.length !== 0 ? (
					<ui.Table width="100%">
						<ui.TableHeader>
							<ui.TableRow>
								<ui.TableHeaderCell>{'Id'}</ui.TableHeaderCell>
								<ui.TableHeaderCell>{'Created'}</ui.TableHeaderCell>
								<ui.TableHeaderCell></ui.TableHeaderCell>
							</ui.TableRow>
						</ui.TableHeader>
						<ui.TableBody>
							{props.models.map(model => (
								<ui.TableRow key={model.id}>
									<ui.TableCell>
										<ui.Link href={`./models/${model.id}/`}>{model.id}</ui.Link>
									</ui.TableCell>
									<ui.TableCell>{model.createdAt}</ui.TableCell>
									<ui.TableCell>
										<form method="post">
											<input name="action" type="hidden" value="delete_model" />
											<input name="modelId" type="hidden" value={model.id} />
											<ui.Button color="var(--red)">{'Delete'}</ui.Button>
										</form>
									</ui.TableCell>
								</ui.TableRow>
							))}
						</ui.TableBody>
					</ui.Table>
				) : (
					<ui.Card>
						<ui.P>{'This repositories has no models.'}</ui.P>
					</ui.Card>
				)}
			</ui.S1>
		</AppLayout>,
	)
}
