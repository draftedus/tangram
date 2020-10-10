import { hydrateLineChart } from '@tangramhq/charts'
import { bootDateWindowSelect } from 'common/date_window_select'

bootDateWindowSelect()
if (document.getElementById('mse')) {
	hydrateLineChart('mse')
}
if (document.getElementById('accuracy')) {
	hydrateLineChart('accuracy')
}
