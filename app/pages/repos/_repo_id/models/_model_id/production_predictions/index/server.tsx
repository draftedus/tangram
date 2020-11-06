import { Props } from './props'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { Fragment, h } from 'preact'

export default (pageInfo: PageInfo, props: Props) => {
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pageInfo={pageInfo}
			selectedItem={ModelSideNavItem.ProductionPredictions}
		>
			<ui.S1>
				<ui.H1>{'Production Predictions'}</ui.H1>
				{props.predictionTable == null ? (
					<ui.P>{'There are no predictions.'}</ui.P>
				) : (
					<Fragment>
						<ui.Form post={true}>
							<div class="search-bar-wrapper">
								<ui.TextField
									autocomplete="off"
									label="Identifier"
									name="identifier"
								/>
								<ui.Button type="submit">{'Lookup'}</ui.Button>
							</div>
						</ui.Form>
						<ui.Table width="100%">
							<ui.TableHeader>
								<ui.TableRow>
									<ui.TableHeaderCell>{'Identifier'}</ui.TableHeaderCell>
									<ui.TableHeaderCell>{'Date'}</ui.TableHeaderCell>
									<ui.TableHeaderCell>{'Output'}</ui.TableHeaderCell>
								</ui.TableRow>
							</ui.TableHeader>
							<ui.TableBody>
								{props.predictionTable.rows.map(prediction => (
									<ui.TableRow key={prediction.identifier}>
										<ui.TableCell>
											<ui.Link href={`./predictions/${prediction.identifier}`}>
												{prediction.identifier}
											</ui.Link>
										</ui.TableCell>
										<ui.TableCell>{prediction.date}</ui.TableCell>
										<ui.TableCell>{prediction.output}</ui.TableCell>
									</ui.TableRow>
								))}
							</ui.TableBody>
						</ui.Table>
						<div class="pagination-buttons">
							<ui.Form>
								{props.pagination.after && (
									<input
										name="after"
										type="hidden"
										value={props.pagination.after}
									/>
								)}
								<ui.Button
									disabled={props.pagination.after === null}
									type="submit"
								>
									{'Newer'}
								</ui.Button>
							</ui.Form>
							<ui.Form>
								{props.pagination.before && (
									<input
										name="before"
										type="hidden"
										value={props.pagination.before}
									/>
								)}
								<ui.Button
									disabled={props.pagination.before === null}
									type="submit"
								>
									{'Older'}
								</ui.Button>
							</ui.Form>
						</div>
					</Fragment>
				)}
			</ui.S1>
		</ModelLayout>,
	)
}
