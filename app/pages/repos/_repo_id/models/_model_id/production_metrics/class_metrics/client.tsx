import { bootClassSelectField } from 'common/class_select'
import { bootDateWindowSelectField } from 'common/time_charts'
import { ui } from 'deps'

bootClassSelectField()
bootDateWindowSelectField()
ui.hydrateLineChart('precision_intervals')
ui.hydrateLineChart('recall_intervals')
ui.hydrateLineChart('f1_intervals')
