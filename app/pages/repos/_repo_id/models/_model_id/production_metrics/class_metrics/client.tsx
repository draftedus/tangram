import { hydrateLineChart } from '@tangramhq/charts'
import { bootClassSelectField } from 'common/class_select'
import { bootDateWindowSelectField } from 'common/date_window_select'

bootClassSelectField()
bootDateWindowSelectField()
hydrateLineChart('precision_intervals')
hydrateLineChart('recall_intervals')
hydrateLineChart('f1_intervals')
