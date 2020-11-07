import { Props } from './props'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { AppLayout } from 'layouts/app_layout'
import { h } from 'preact'

export default (pageInfo: PageInfo, props: Props) => {
	return renderPage(
		<AppLayout info={props.appLayoutInfo} pageInfo={pageInfo}>
			<ui.S1>
				<ui.SpaceBetween>
					<ui.H1>{props.title}</ui.H1>
					<ui.Button href="./models/new">{'Upload New Version'}</ui.Button>
				</ui.SpaceBetween>
				<ui.S2>
					{/* <ui.H2>{'Models'}</ui.H2> */}
					{props.models.length > 0 ? (
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
											<ui.Link href={`./models/${model.id}/`}>
												{model.id}
											</ui.Link>
										</ui.TableCell>
										<ui.TableCell>{model.createdAt}</ui.TableCell>
										<ui.TableCell>
											<form method="post">
												<input
													name="action"
													type="hidden"
													value="delete_model"
												/>
												<input name="model_id" type="hidden" value={model.id} />
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
				</ui.S2>
			</ui.S1>
		</AppLayout>,
	)
}
