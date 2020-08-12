import { bootClassSelectField } from 'common/class_select'
import { ui } from 'deps'

bootClassSelectField()
if (
	document.getElementById('number_intervals') &&
	document.getElementById('number_overall')
) {
	ui.hydrateBoxChart('number_intervals')
	ui.hydrateBoxChart('number_overall')
}
if (
	document.getElementById('enum_intervals') &&
	document.getElementById('enum_overall')
) {
	ui.hydrateBarChart('enum_intervals')
	ui.hydrateBarChart('enum_overall')
}
if (document.getElementById('text_overall')) {
	ui.hydrateBarChart('text_overall')
}
