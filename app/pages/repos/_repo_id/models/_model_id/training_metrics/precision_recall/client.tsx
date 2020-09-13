import { hydrateLineChart } from '@tangramhq/charts'
import { bootClassSelectField } from 'common/class_select'

bootClassSelectField()
hydrateLineChart('parametric_pr')
hydrateLineChart('non_parametric_pr')
