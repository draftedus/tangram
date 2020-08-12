import { ui } from 'deps'

if (document.getElementById('feature_importances')) {
	ui.hydrateBarChart('feature_importances')
}
if (document.getElementById('feature_weights')) {
	ui.hydrateBarChart('feature_weights')
}
