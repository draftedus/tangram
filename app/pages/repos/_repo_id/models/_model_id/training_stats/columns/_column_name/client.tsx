import { ui } from 'deps'

if (document.getElementById('enum_histogram')) {
	ui.hydrateBarChart('enum_histogram')
}
if (document.getElementById('number_histogram')) {
	ui.hydrateBarChart('number_histogram')
}
if (document.getElementById('text_histogram')) {
	ui.hydrateBarChart('text_histogram')
}
