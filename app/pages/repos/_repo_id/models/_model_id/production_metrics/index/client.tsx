import { hydrateLineChart } from '@tangramhq/charts'
import { bootDateWindowSelectField } from 'common/date_window_select'

bootDateWindowSelectField()
if (document.getElementById('mse')) {
	hydrateLineChart('mse')
}
if (document.getElementById('accuracy')) {
	hydrateLineChart('accuracy')
}
