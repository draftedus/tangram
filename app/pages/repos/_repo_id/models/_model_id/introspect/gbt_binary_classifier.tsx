import { GBTFeatureImportances } from './gbt'
import { h } from 'deps'

type Props = {
	featureImportances: Array<[string, number]>
}

export function GBTBinaryClassifierModelPage(props: Props) {
	return <GBTFeatureImportances values={props.featureImportances} />
}
