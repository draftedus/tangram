import { hydrateLineChart } from '@tangramhq/charts'
import { selectSubmitOnChange } from '@tangramhq/ui'

selectSubmitOnChange('date-window-select-field')
selectSubmitOnChange('class-select-field')
hydrateLineChart('precision_intervals')
hydrateLineChart('recall_intervals')
hydrateLineChart('f1_intervals')
