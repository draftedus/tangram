import { TreeFeatureImportances } from './tree'
import { h } from 'deps'

type Props = {
	featureImportances: Array<[string, number]>
}

export function TreeRegressorModelPage(props: Props) {
	return <TreeFeatureImportances values={props.featureImportances} />
}
