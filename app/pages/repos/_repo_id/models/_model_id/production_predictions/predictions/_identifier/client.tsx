import { hydrateFeatureContributionsChart } from '@tangramhq/charts'

if (document.getElementById('regression_feature_contributions')) {
	hydrateFeatureContributionsChart('regression_feature_contributions')
}
if (document.getElementById('binary_classification_feature_contributions')) {
	hydrateFeatureContributionsChart(
		'binary_classification_feature_contributions',
	)
}
if (
	document.getElementById('multiclass_classification_feature_contributions')
) {
	hydrateFeatureContributionsChart(
		'multiclass_classification_feature_contributions',
	)
}
