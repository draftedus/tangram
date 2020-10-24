import { BinaryClassifierProductionMetricsIndexPage } from './binary_classifier'
import { MulticlassClassifierProductionMetricsIndexPage } from './multiclass_classifier'
import {
	BinaryClassifierProps,
	MulticlassClassifierProps,
	RegressorProps,
} from './props'
import { RegressorProductionMetricsPage } from './regressor'
import { PageInfo } from '@tangramhq/pinwheel'
import { renderPage } from 'common/render'
import {
	ModelLayout,
	ModelLayoutInfo,
	ModelSideNavItem,
} from 'layouts/model_layout'
import { h } from 'preact'

export type Props = {
	inner: Inner
	modelLayoutInfo: ModelLayoutInfo
}

export type Inner =
	| {
			type: Type.Regressor
			value: RegressorProps
	  }
	| {
			type: Type.BinaryClassifier
			value: BinaryClassifierProps
	  }
	| {
			type: Type.MulticlassClassifier
			value: MulticlassClassifierProps
	  }

export enum Type {
	Regressor = 'regressor',
	BinaryClassifier = 'binary_classifer',
	MulticlassClassifier = 'multiclass_classifier',
}

export default (pageInfo: PageInfo, props: Props) => {
	let inner
	switch (props.inner.type) {
		case Type.Regressor: {
			inner = <RegressorProductionMetricsPage {...props.inner.value} />
			break
		}
		case Type.BinaryClassifier: {
			inner = (
				<BinaryClassifierProductionMetricsIndexPage {...props.inner.value} />
			)
			break
		}
		case Type.MulticlassClassifier: {
			inner = (
				<MulticlassClassifierProductionMetricsIndexPage
					{...props.inner.value}
				/>
			)
			break
		}
	}
	return renderPage(
		<ModelLayout
			clientJsSrc={pageInfo.clientJsSrc}
			cssSrcs={pageInfo.cssSrcs}
			modelLayoutInfo={props.modelLayoutInfo}
			preloadJsSrcs={pageInfo.preloadJsSrcs}
			selectedItem={ModelSideNavItem.ProductionMetrics}
		>
			{inner}
		</ModelLayout>,
	)
}
