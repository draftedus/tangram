import { Explanations } from './explanations'
import { Introspection } from './introspection'
import { ProductionMetrics } from './production_metrics'
import { ProductionStats } from './production_stats'
import { Training } from './training'
import { TrainingMetrics } from './training_metrics'
import { TrainingStats } from './training_stats'
import { Tuning } from './tuning'
import { hydrateComponent } from '@tangramhq/pinwheel'

hydrateComponent('training', Training)
hydrateComponent('training-stats', TrainingStats)
hydrateComponent('training-metrics', TrainingMetrics)
hydrateComponent('introspection', Introspection)
hydrateComponent('tuning', Tuning)
hydrateComponent('explanations', Explanations)
hydrateComponent('production-stats', ProductionStats)
hydrateComponent('production-metrics', ProductionMetrics)
