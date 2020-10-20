import { TreeMulticlassClassifierProps } from './props'
import { TreeFeatureImportances } from './tree_feature_importances'
import { h } from 'preact'

export function TreeMulticlassClassifierModelPage(
	props: TreeMulticlassClassifierProps,
) {
	return <TreeFeatureImportances values={props.featureImportances} />
}
