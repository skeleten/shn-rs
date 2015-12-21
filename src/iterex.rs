use std::collections::VecDeque;

pub trait IteratorEx<I> {
	fn reverse(self) -> ReverseIterator<I>;
}

impl<T, I> IteratorEx<I> for T where T: Iterator<Item = I> {
	fn reverse(self) -> ReverseIterator<I> {
		let deque = self.collect();
		ReverseIterator {
			items: deque
		}
	}
}

pub struct ReverseIterator<T> {
	items: VecDeque<T>,
}

impl<T> Iterator for ReverseIterator<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		self.items.pop_back()
	}
}
