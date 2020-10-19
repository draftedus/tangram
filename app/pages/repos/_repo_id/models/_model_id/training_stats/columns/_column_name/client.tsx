import { hydrateBarChart, hydrateBoxChart } from '@tangramhq/charts'

if (document.getElementById('enum_histogram')) {
	hydrateBarChart('enum_histogram')
}
if (document.getElementById('number_quantiles')) {
	hydrateBoxChart('number_quantiles')
}
if (document.getElementById('number_histogram')) {
	hydrateBarChart('number_histogram')
}
if (document.getElementById('token_histogram')) {
	hydrateBarChart('token_histogram')
}
