import { TreeFeatureImportances } from './tree'
import { h } from 'deps'

type Props = {
	featureImportances: Array<[string, number]>
}

export function TreeMulticlassClassifierModelPage(props: Props) {
	return <TreeFeatureImportances values={props.featureImportances} />
}
