import { bootClassSelectField } from 'common/class_select'
import { bootDateWindowSelectField } from 'common/time_charts'
import { ui } from 'deps'

bootClassSelectField()
bootDateWindowSelectField()
if (document.getElementById('number_intervals')) {
	ui.hydrateBoxChart('number_intervals')
}
if (document.getElementById('number_overall')) {
	ui.hydrateBoxChart('number_overall')
}
if (document.getElementById('enum_overall')) {
	ui.hydrateBarChart('enum_overall')
}
if (document.getElementById('text_overall')) {
	ui.hydrateBarChart('text_overall')
}
