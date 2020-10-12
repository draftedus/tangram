import { hydrateBarChart, hydrateBoxChart } from '@tangramhq/charts'
import { bootClassSelect } from 'common/class_select_field'
import { bootDateWindowSelect } from 'common/date_window_select_field'

bootClassSelect()
bootDateWindowSelect()
if (document.getElementById('number_intervals')) {
	hydrateBoxChart('number_intervals')
}
if (document.getElementById('number_overall')) {
	hydrateBoxChart('number_overall')
}
if (document.getElementById('enum_overall')) {
	hydrateBarChart('enum_overall')
}
if (document.getElementById('text_overall')) {
	hydrateBarChart('text_overall')
}
