import { hydrateBarChart } from '@tangramhq/charts'
import { bootClassSelect } from 'common/class_select_field'
import { bootDateWindowSelect } from 'common/date_window_select_field'

bootDateWindowSelect()
bootClassSelect()
hydrateBarChart('precision_intervals')
hydrateBarChart('recall_intervals')
hydrateBarChart('f1_intervals')
