use std::cell::RefCell;
use std::rc::Rc;

pub struct Pool<T> {
	create_item: Box<dyn Fn() -> T>,
	max_items: usize,
	state: Rc<RefCell<State<T>>>,
}

struct State<T> {
	n_items_outstanding: usize,
	available_items: Vec<T>,
}

impl<T> Pool<T> {
	pub fn new(max_items: usize, create_item: Box<dyn Fn() -> T>) -> Self {
		Self {
			create_item,
			max_items,
			state: Rc::new(RefCell::new(State {
				n_items_outstanding: 0,
				available_items: Vec::new(),
			})),
		}
	}

	pub fn get(&mut self) -> Option<PoolGuard<T>> {
		let mut state = self.state.borrow_mut();
		if let Some(item) = state.available_items.pop() {
			state.n_items_outstanding += 1;
			Some(PoolGuard {
				item: Some(item),
				state: self.state.clone(),
			})
		} else if state.n_items_outstanding < self.max_items {
			state.n_items_outstanding += 1;
			let item = (self.create_item)();
			Some(PoolGuard {
				item: Some(item),
				state: self.state.clone(),
			})
		} else {
			None
		}
	}
}

pub struct PoolGuard<T> {
	item: Option<T>,
	state: Rc<RefCell<State<T>>>,
}

impl<T> std::ops::Deref for PoolGuard<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		self.item.as_ref().unwrap()
	}
}

impl<T> std::ops::DerefMut for PoolGuard<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.item.as_mut().unwrap()
	}
}

impl<T> Drop for PoolGuard<T> {
	fn drop(&mut self) {
		let mut state = self.state.borrow_mut();
		state.available_items.push(self.item.take().unwrap());
		state.n_items_outstanding -= 1;
	}
}
