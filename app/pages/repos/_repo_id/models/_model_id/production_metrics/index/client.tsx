import { bootDateWindowSelectField } from 'common/time'
import { ui } from 'deps'

bootDateWindowSelectField()
if (document.getElementById('mse')) {
	ui.hydrateLineChart('mse')
}
if (document.getElementById('accuracy')) {
	ui.hydrateLineChart('accuracy')
}
