import { bootDateWindowSelectField } from 'common/time_charts'
import { ui } from 'deps'

bootDateWindowSelectField()
ui.hydrateLineChart('mse')
ui.hydrateLineChart('accuracy')
