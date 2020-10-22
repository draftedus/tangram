import { FeatureImportancesTable } from './feature_importances_table'
import { LinearRegressorProps } from './props'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export function LinearRegressorTrainingImportancesPage(
	props: LinearRegressorProps,
) {
	return (
		<ui.S1>
			<ui.H1>{'Training Feature Importances'}</ui.H1>
			<FeatureImportancesTable values={props.featureImportances} />
		</ui.S1>
	)
}
