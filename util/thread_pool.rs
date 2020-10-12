use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use once_cell::sync::Lazy;

pub static GLOBAL_THREAD_POOL: Lazy<ThreadPool> = Lazy::new(|| ThreadPool::new(16));

pub struct ThreadPool {
	threads: Vec<Thread>,
}

impl ThreadPool {
	pub fn new(size: usize) -> ThreadPool {
		ThreadPool {
			threads: (0..size).map(|_| Thread::new()).collect(),
		}
	}

	pub fn execute<'s, F>(&self, f: Vec<F>)
	where
		F: FnOnce() + Send + 's,
	{
		let mut done_receivers = Vec::with_capacity(f.len());
		for (i, f) in f.into_iter().enumerate() {
			// This is safe because this function blocks until all tasks have completed.
			let f = unsafe {
				std::mem::transmute::<
					Box<dyn FnOnce() + Send + 's>,
					Box<dyn FnOnce() + Send + 'static>,
				>(Box::new(f))
			};
			let n_threads = self.threads.len();
			let thread = unsafe { self.threads.get_unchecked(i % n_threads) };
			let done_receiver = thread.spawn(f);
			done_receivers.push(done_receiver);
		}
		for done_receiver in done_receivers {
			done_receiver.recv().unwrap();
		}
	}
}

#[derive(Debug)]
struct Thread {
	task_sender: Option<Sender<Task>>,
	join_handle: Option<std::thread::JoinHandle<()>>,
}

impl Thread {
	fn new() -> Thread {
		let (task_sender, task_receiver) = unbounded::<Task>();
		let join_handle = std::thread::spawn(move || loop {
			match task_receiver.recv() {
				Err(_) => break,
				Ok(task) => {
					(task.f)();
					task.done_sender.send(()).unwrap();
				}
			};
		});
		Thread {
			join_handle: Some(join_handle),
			task_sender: Some(task_sender),
		}
	}

	fn spawn(&self, f: Box<dyn FnOnce() + Send + 'static>) -> Receiver<()> {
		let (done_sender, done_receiver) = bounded::<()>(1);
		let task = Task { f, done_sender };
		self.task_sender.as_ref().unwrap().send(task).unwrap();
		done_receiver
	}
}

impl Drop for Thread {
	fn drop(&mut self) {
		self.task_sender.take().unwrap();
		self.join_handle.take().unwrap().join().unwrap();
	}
}

struct Task {
	f: Box<dyn FnOnce() + Send + 'static>,
	done_sender: Sender<()>,
}
