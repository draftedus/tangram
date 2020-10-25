import { FeatureImportancesTable } from './feature_importances_table'
import { TreeMulticlassClassifierProps } from './props'
import * as ui from '@tangramhq/ui'
import { h } from 'preact'

export function TreeMulticlassClassifierTrainingImportancesPage(
	props: TreeMulticlassClassifierProps,
) {
	return (
		<ui.S1>
			<ui.H1>{'Training Feature Importances'}</ui.H1>
			<FeatureImportancesTable featureImportances={props.featureImportances} />
		</ui.S1>
	)
}