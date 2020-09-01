import { bootDateWindowSelectField } from 'common/time'
import { ui } from 'deps'

bootDateWindowSelectField()
ui.hydrateBarChart('prediction_count')
if (
	document.getElementById('quantiles_overall') &&
	document.getElementById('quantiles_intervals')
) {
	ui.hydrateBoxChart('quantiles_overall')
	ui.hydrateBoxChart('quantiles_intervals')
}
if (
	document.getElementById('histogram_overall') &&
	document.getElementById('histogram_intervals')
) {
	ui.hydrateBarChart('histogram_overall')
	ui.hydrateBarChart('histogram_intervals')
}
