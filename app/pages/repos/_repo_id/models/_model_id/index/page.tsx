import { ClassifierIndexPage, Props as ClassifierProps } from './classifier'
import { RegressorIndexPage, Props as RegressorProps } from './regressor'
import { h } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

export type Props = {
	inner: Inner
	modelLayout: ModelLayoutProps
}

export enum Type {
	Regressor = 'regressor',
	Classifier = 'classifier',
}

export type Inner =
	| { type: Type.Regressor; value: RegressorProps }
	| { type: Type.Classifier; value: ClassifierProps }

export default function ModelIndexPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case Type.Regressor: {
			inner = <RegressorIndexPage {...props.inner.value} />
			break
		}
		case Type.Classifier: {
			inner = <ClassifierIndexPage {...props.inner.value} />
			break
		}
	}
	return <ModelLayout {...props.modelLayout}>{inner}</ModelLayout>
}
