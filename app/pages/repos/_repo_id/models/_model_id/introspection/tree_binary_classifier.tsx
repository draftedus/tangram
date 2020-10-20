import { TreeBinaryClassifierProps } from './props'
import { TreeFeatureImportances } from './tree_feature_importances'
import { h } from 'preact'

export function TreeBinaryClassifierModelPage(
	props: TreeBinaryClassifierProps,
) {
	return <TreeFeatureImportances values={props.featureImportances} />
}
