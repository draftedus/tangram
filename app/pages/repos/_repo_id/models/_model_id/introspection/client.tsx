import { hydrateBarChart } from '@tangramhq/charts'

if (document.getElementById('feature_importances')) {
	hydrateBarChart('feature_importances')
}
if (document.getElementById('feature_weights')) {
	hydrateBarChart('feature_weights')
}
