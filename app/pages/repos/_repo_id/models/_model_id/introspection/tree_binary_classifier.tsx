import { TreeFeatureImportances } from './tree_feature_importances'
import { h } from 'preact'

type Props = {
	featureImportances: Array<[string, number]>
}

export function TreeBinaryClassifierModelPage(props: Props) {
	return <TreeFeatureImportances values={props.featureImportances} />
}
