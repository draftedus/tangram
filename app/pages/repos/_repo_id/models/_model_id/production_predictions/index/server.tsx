import { Props } from './props'
import * as ui from '@tangramhq/ui'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { Fragment, h } from 'preact'

export default function ProductionPredictionIndexPage(props: Props) {
	return renderPage(
		<ModelLayout
			info={props.modelLayoutInfo}
			pinwheelInfo={props.pinwheelInfo}
			selectedItem={ModelSideNavItem.ProductionPredictions}
		>
			<ui.S1>
				<ui.H1>{'Production Predictions'}</ui.H1>
				{/* <ui.Form>
					<div class="search-bar-wrapper">
						<ui.TextField label="Identifier" name="identifier"></ui.TextField>
						<ui.Button type="submit">{'Lookup'}</ui.Button>
					</div>
				</ui.Form> */}
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
						{props.pagination.after ? (
							<Fragment>
								<input
									name="after"
									type="hidden"
									value={props.pagination.after}
								/>
								<ui.Button type="submit">{'Newer'}</ui.Button>
							</Fragment>
						) : (
							<ui.Button disabled={true} type="submit">
								{'Newer'}
							</ui.Button>
						)}
					</ui.Form>
					<ui.Form>
						{props.pagination.before ? (
							<Fragment>
								<input
									name="before"
									type="hidden"
									value={props.pagination.before}
								/>
								<ui.Button type="submit">{'Older'}</ui.Button>
							</Fragment>
						) : (
							<ui.Button disabled={true} type="submit">
								{'Older'}
							</ui.Button>
						)}
					</ui.Form>
				</div>
			</ui.S1>
		</ModelLayout>,
	)
}
