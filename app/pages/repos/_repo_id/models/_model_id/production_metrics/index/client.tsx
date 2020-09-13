import { hydrateLineChart } from '@tangramhq/charts'
import { bootDateWindowSelectField } from 'common/time'

bootDateWindowSelectField()
if (document.getElementById('mse')) {
	hydrateLineChart('mse')
}
if (document.getElementById('accuracy')) {
	hydrateLineChart('accuracy')
}
