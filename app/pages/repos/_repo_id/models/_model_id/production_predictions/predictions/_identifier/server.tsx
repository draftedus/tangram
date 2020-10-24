import { InnerType, Props } from './props'
import { PageInfo } from '@tangramhq/pinwheel'
import * as ui from '@tangramhq/ui'
import { PredictionResult } from 'common/prediction_result'
import { renderPage } from 'common/render'
import { ModelLayout, ModelSideNavItem } from 'layouts/model_layout'
import { Fragment, h } from 'preact'

export default (pageInfo: PageInfo, props: Props) => {
	return renderPage(
		<ModelLayout
			clientJsSrc={pageInfo.clientJsSrc}
			cssSrcs={pageInfo.cssSrcs}
			modelLayoutInfo={props.modelLayoutInfo}
			preloadJsSrcs={pageInfo.preloadJsSrcs}
			selectedItem={ModelSideNavItem.ProductionPredictions}
		>
			<ui.S1>
				<ui.H1>{'Prediction'}</ui.H1>
				{props.inner.type == InnerType.NotFound ? (
					<ui.P>
						<ui.Alert level={ui.Level.Danger}>
							{'Prediction with identifier '}
							<b>{props.identifier}</b>
							{' not found'}
						</ui.Alert>
					</ui.P>
				) : (
					<Fragment>
						<ui.Table width="100%">
							<ui.TableHeader>
								<ui.TableRow>
									<ui.TableHeaderCell>{'Identifier'}</ui.TableHeaderCell>
									<ui.TableHeaderCell>{'Date'}</ui.TableHeaderCell>
								</ui.TableRow>
							</ui.TableHeader>
							<ui.TableBody>
								<ui.TableRow>
									<ui.TableCell>{props.identifier}</ui.TableCell>
									<ui.TableCell>{props.inner.value.date}</ui.TableCell>
								</ui.TableRow>
							</ui.TableBody>
						</ui.Table>
						<PredictionResult
							inputTable={props.inner.value.inputTable}
							prediction={props.inner.value.prediction}
						/>
					</Fragment>
				)}
			</ui.S1>
		</ModelLayout>,
	)
}
