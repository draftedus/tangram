import { TreeRegressorProps } from './props'
import { TreeFeatureImportances } from './tree_feature_importances'
import { h } from 'preact'

export function TreeRegressorModelPage(props: TreeRegressorProps) {
	return <TreeFeatureImportances values={props.featureImportances} />
}
