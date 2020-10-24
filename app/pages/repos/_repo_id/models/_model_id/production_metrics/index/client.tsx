import { hydrateBarChart } from '@tangramhq/charts'
import { bootDateWindowSelect } from 'common/date_window_select_field'

bootDateWindowSelect()
if (document.getElementById('mse')) {
	hydrateBarChart('mse')
}
if (document.getElementById('accuracy')) {
	hydrateBarChart('accuracy')
}
