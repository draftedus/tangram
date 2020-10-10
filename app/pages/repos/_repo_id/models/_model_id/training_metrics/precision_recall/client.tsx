import { hydrateLineChart } from '@tangramhq/charts'
import { bootClassSelect } from 'common/class_select'

bootClassSelect()
hydrateLineChart('parametric_pr')
hydrateLineChart('non_parametric_pr')
