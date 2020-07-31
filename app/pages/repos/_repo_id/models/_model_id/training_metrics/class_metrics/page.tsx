import {
	BinaryClassifierClassMetricsPage,
	Props as BinaryClassifierProps,
} from './binary_classifier'
import {
	MulticlassClassifierClassMetricsPage,
	Props as MulticlassClassifierProps,
} from './multiclass_classifier'
import { h, ui } from 'deps'
import { ModelLayout, ModelLayoutProps } from 'layouts/model_layout'

export type Props = {
	inner: Inner
	modelLayoutProps: ModelLayoutProps
}

export enum ModelType {
	BinaryClassifier = 'BinaryClassifier',
	MulticlassClassifier = 'MulticlassClassifier',
}

export type Inner =
	| {
			type: ModelType.BinaryClassifier
			value: BinaryClassifierProps
	  }
	| {
			type: ModelType.MulticlassClassifier
			value: MulticlassClassifierProps
	  }

export default function ClassMetricsPage(props: Props) {
	let inner
	switch (props.inner.type) {
		case ModelType.BinaryClassifier:
			inner = <BinaryClassifierClassMetricsPage {...props.inner.value} />
			break
		case ModelType.MulticlassClassifier:
			inner = <MulticlassClassifierClassMetricsPage {...props.inner.value} />
			break
	}
	return <ModelLayout {...props.modelLayoutProps}>{inner}</ModelLayout>
}

type ClassSelectProps = {
	class: string
	classes: string[]
}

export function ClassSelect(props: ClassSelectProps) {
	return (
		<div>
			<ui.Form>
				<ui.SelectField
					label="class"
					name="class"
					options={props.classes}
					value={props.class}
				/>
				<ui.Button>Submit</ui.Button>
			</ui.Form>
		</div>
	)
}
