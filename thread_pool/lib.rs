#[macro_export]
macro_rules! pzip {
	($($e:expr),* $(,)*) => {
		rayon::iter::IntoParallelIterator::into_par_iter(($($e,)*))
	};
}

// struct ThreadPool;

// impl ThreadPool {
// 	fn execute<F, R>(f: Vec<F>) -> Vec<R>
// 	where
// 		F: FnOnce() -> R,
// 	{
// 		f.into_iter().map(|f| f()).collect()
// 	}
// }
