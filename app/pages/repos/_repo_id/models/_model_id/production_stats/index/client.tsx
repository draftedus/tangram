import { hydrateBarChart, hydrateBoxChart } from '@tangramhq/charts'
import { bootDateWindowSelectField } from 'common/time'

bootDateWindowSelectField()
hydrateBarChart('prediction_count')
if (
	document.getElementById('quantiles_overall') &&
	document.getElementById('quantiles_intervals')
) {
	hydrateBoxChart('quantiles_overall')
	hydrateBoxChart('quantiles_intervals')
}
if (
	document.getElementById('histogram_overall') &&
	document.getElementById('histogram_intervals')
) {
	hydrateBarChart('histogram_overall')
	hydrateBarChart('histogram_intervals')
}
