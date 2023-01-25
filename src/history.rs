#![allow(dead_code)]

use std::collections::VecDeque;

use parking_lot::{lock_api::RwLockReadGuard, RawRwLock, RwLock};

pub struct History<T> {
  deque: RwLock<VecDeque<T>>,
  limit: usize,
}

impl<T> History<T> {
  pub fn new(limit: usize) -> History<T> {
    History {
      deque: RwLock::new(VecDeque::new()),
      limit,
    }
  }

  pub fn with_capacity(limit: usize, capacity: usize) -> History<T> {
    History {
      deque: RwLock::new(VecDeque::with_capacity(capacity)),
      limit,
    }
  }

  pub fn push(&self, value: T) -> Option<T>
  where
    T: PartialEq,
  {
    let mut deque = self.deque.write();
    for (idx, v) in deque.iter().enumerate() {
      if *v == value {
        deque.remove(idx);
        break;
      }
    }

    let last = if deque.len() >= self.limit {
      deque.pop_back()
    } else {
      None
    };
    deque.push_front(value);
    last
  }

  pub fn clear(&self) {
    let mut deque = self.deque.write();
    deque.clear();
  }

  pub fn len(&self) -> usize {
    let deque = self.deque.read();
    deque.len()
  }

  pub fn is_empty(&self) -> bool {
    let deque = self.deque.read();
    deque.is_empty()
  }

  pub fn read(&self) -> RwLockReadGuard<RawRwLock, VecDeque<T>> {
    self.deque.read()
  }

  #[inline]
  pub fn into_inner(self) -> VecDeque<T> {
    self.deque.into_inner()
  }

  #[inline]
  pub fn into_iter(self) -> std::collections::vec_deque::IntoIter<T> {
    self.into_inner().into_iter()
  }

  #[inline]
  fn contains(&self, value: &T) -> bool
  where
    T: PartialEq,
  {
    let deque = self.deque.read();
    deque.contains(value)
  }
}

#[cfg(test)]
mod tests {
  use super::History;

  #[test]
  fn test() {
    let his = History::with_capacity(10, 10);
    for i in 1..=10 {
      his.push(i.to_string());
    }
    assert_eq!(his.len(), 10);
    his.push("11".to_string());
    assert_eq!(his.len(), 10);
    let mut his = his.into_inner();
    for i in (2..=11).rev() {
      let first = his.pop_front();
      assert!(first.unwrap().to_string() == i.to_string());
    }
  }
}
