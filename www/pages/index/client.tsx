import { boot } from '../../deps'
import { Explanations } from './explanations'
import { Introspection } from './introspection'
import { ProductionMetrics } from './production_metrics'
import { ProductionStats } from './production_stats'
import { Training } from './training'
import { TrainingMetrics } from './training_metrics'
import { TrainingStats } from './training_stats'
import { Tuning } from './tuning'

boot('training', Training)
boot('training-stats', TrainingStats)
boot('training-metrics', TrainingMetrics)
boot('introspection', Introspection)
boot('tuning', Tuning)
boot('explanations', Explanations)
boot('production-stats', ProductionStats)
boot('production-metrics', ProductionMetrics)
