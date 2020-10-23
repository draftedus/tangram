import { FeatureImportancesTable } from './feature_importances_table'
import { TreeRegressorProps } from './props'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export function TreeRegressorTrainingImportancesPage(
	props: TreeRegressorProps,
) {
	return (
		<ui.S1>
			<ui.H1>{'Training Feature Importances'}</ui.H1>
			<FeatureImportancesTable featureImportances={props.featureImportances} />
		</ui.S1>
	)
}
