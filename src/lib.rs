use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

/// An immutable-style ordered map with a movable cursor.
///
/// Each mutation returns a new `LinkedMap`, leaving the original value unchanged.
/// Keys must be unique.
#[derive(Debug)]
pub struct LinkedMap<K, V> {
    entries: BTreeMap<K, Arc<V>>,
    order: Vec<K>,
    current: Option<K>,
}

impl<K, V> Clone for LinkedMap<K, V>
where
    K: Clone,
{
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            order: self.order.clone(),
            current: self.current.clone(),
        }
    }
}

impl<K, V> Default for LinkedMap<K, V> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            order: Vec::new(),
            current: None,
        }
    }
}

impl<K, V> LinkedMap<K, V>
where
    K: Ord,
{
    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).map(|value| value.as_ref())
    }

    pub fn current_key(&self) -> Option<&K> {
        self.current.as_ref()
    }

    pub fn current_value(&self) -> Option<&V> {
        self.current
            .as_ref()
            .and_then(|key| self.entries.get(key))
            .map(|value| value.as_ref())
    }

    pub fn first_key(&self) -> Option<&K> {
        self.order.first()
    }

    pub fn last_key(&self) -> Option<&K> {
        self.order.last()
    }

    pub fn first(&self) -> Option<&V> {
        self.first_key()
            .and_then(|key| self.entries.get(key))
            .map(|value| value.as_ref())
    }

    pub fn last(&self) -> Option<&V> {
        self.last_key()
            .and_then(|key| self.entries.get(key))
            .map(|value| value.as_ref())
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            map: self,
            front: 0,
            back: self.order.len(),
        }
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }

    pub fn reduce<T, F>(&self, init: T, mut f: F) -> T
    where
        F: FnMut(T, &V, &K) -> T,
    {
        self.order.iter().fold(init, |acc, key| {
            let value = self
                .entries
                .get(key)
                .expect("linked map invariant violation: missing value for key");
            f(acc, value.as_ref(), key)
        })
    }

    pub fn reduce_right<T, F>(&self, init: T, mut f: F) -> T
    where
        F: FnMut(T, &V, &K) -> T,
    {
        self.order.iter().rev().fold(init, |acc, key| {
            let value = self
                .entries
                .get(key)
                .expect("linked map invariant violation: missing value for key");
            f(acc, value.as_ref(), key)
        })
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&V, &K),
    {
        for key in &self.order {
            let value = self
                .entries
                .get(key)
                .expect("linked map invariant violation: missing value for key");
            f(value.as_ref(), key);
        }
    }
}

impl<K, V> LinkedMap<K, V> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<K, V> LinkedMap<K, V>
where
    K: Clone + Ord,
{
    pub fn from_entries<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut entries = BTreeMap::new();
        let mut order = Vec::new();
        let mut current = None;

        for (key, value) in iter {
            if entries.contains_key(&key) {
                panic!("cannot insert duplicate key");
            }

            if current.is_none() {
                current = Some(key.clone());
            }

            order.push(key.clone());
            entries.insert(key, Arc::new(value));
        }

        Self {
            entries,
            order,
            current,
        }
    }

    pub fn next(&self) -> Self {
        let Some(current) = self.current.as_ref() else {
            return Self {
                entries: self.entries.clone(),
                order: self.order.clone(),
                current: self.current.clone(),
            };
        };

        let current_index = self.index_of_key_or_panic(current);
        Self {
            entries: self.entries.clone(),
            order: self.order.clone(),
            current: self.order.get(current_index + 1).cloned(),
        }
    }

    pub fn prev(&self) -> Self {
        let Some(current) = self.current.as_ref() else {
            return Self {
                entries: self.entries.clone(),
                order: self.order.clone(),
                current: self.current.clone(),
            };
        };

        let current_index = self.index_of_key_or_panic(current);
        Self {
            entries: self.entries.clone(),
            order: self.order.clone(),
            current: current_index
                .checked_sub(1)
                .and_then(|index| self.order.get(index))
                .cloned(),
        }
    }

    pub fn move_to(&self, key: &K) -> Self {
        if !self.contains_key(key) {
            return Self {
                entries: self.entries.clone(),
                order: self.order.clone(),
                current: self.current.clone(),
            };
        }

        Self {
            entries: self.entries.clone(),
            order: self.order.clone(),
            current: Some(key.clone()),
        }
    }

    pub fn move_to_start(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            order: self.order.clone(),
            current: self.first_key().cloned(),
        }
    }

    pub fn move_to_end(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            order: self.order.clone(),
            current: self.last_key().cloned(),
        }
    }

    pub fn map_values<U, F>(&self, mut f: F) -> LinkedMap<K, U>
    where
        F: FnMut(&V, &K) -> U,
    {
        let mut entries = BTreeMap::new();

        for key in &self.order {
            let value = self
                .entries
                .get(key)
                .expect("linked map invariant violation: missing value for key");
            entries.insert(key.clone(), Arc::new(f(value.as_ref(), key)));
        }

        LinkedMap {
            entries,
            order: self.order.clone(),
            current: self.current.clone(),
        }
    }

    fn index_of_key_or_panic(&self, key: &K) -> usize {
        self.order
            .iter()
            .position(|candidate| candidate == key)
            .unwrap_or_else(|| panic!("item was not found"))
    }
}

impl<K, V> LinkedMap<K, V>
where
    K: Clone + Ord,
{
    pub fn push(&self, value: V, key: K) -> Self {
        self.insert_pair_at(self.order.len(), key, value)
    }

    pub fn push_many<I>(&self, iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        iter.into_iter()
            .fold(self.clone(), |map, (key, value)| map.push(value, key))
    }

    pub fn pop(&self) -> Self {
        match self.last_key() {
            Some(key) => self.remove(key),
            None => self.clone(),
        }
    }

    pub fn pop_many(&self, count: usize) -> Self {
        (0..count).fold(self.clone(), |map, _| map.pop())
    }

    pub fn prepend(&self, value: V, key: K) -> Self {
        self.insert_pair_at(0, key, value)
    }

    pub fn unshift<I>(&self, iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let pairs: Vec<_> = iter.into_iter().collect();
        pairs
            .into_iter()
            .rev()
            .fold(self.clone(), |map, (key, value)| map.prepend(value, key))
    }

    pub fn shift(&self) -> Self {
        match self.first_key() {
            Some(key) => self.remove(key),
            None => self.clone(),
        }
    }

    pub fn concat(&self, other: &Self) -> Self {
        other.order.iter().fold(self.clone(), |map, key| {
            let value = other
                .entries
                .get(key)
                .expect("linked map invariant violation: missing value for key")
                .clone();
            map.insert_shared_at(map.order.len(), key.clone(), value)
        })
    }

    pub fn set(&self, key: &K, value: V) -> Self {
        if !self.contains_key(key) {
            panic!("item was not found");
        }

        let mut next = self.clone();
        next.entries.insert(key.clone(), Arc::new(value));
        next
    }

    pub fn update<F>(&self, key: &K, f: F) -> Self
    where
        F: FnOnce(&V) -> V,
    {
        let value = self
            .entries
            .get(key)
            .unwrap_or_else(|| panic!("item was not found"));

        self.set(key, f(value.as_ref()))
    }

    pub fn remove(&self, key: &K) -> Self {
        if !self.contains_key(key) {
            return self.clone();
        }

        let mut next = self.clone();
        next.entries.remove(key);
        next.order.retain(|candidate| candidate != key);

        if next.current.as_ref() == Some(key) {
            next.current = None;
        }

        next
    }

    pub fn delete(&self, key: &K) -> Self {
        self.remove(key)
    }

    pub fn swap(&self, key1: &K, key2: &K) -> Self {
        let pos1 = self.index_of_key_or_panic(key1);
        let pos2 = self.index_of_key_or_panic(key2);

        if pos1 == pos2 {
            return self.clone();
        }

        let mut next = self.clone();
        next.order.swap(pos1, pos2);
        next
    }

    pub fn insert_after(&self, after: &K, value: V, key: K) -> Self {
        let index = self.index_of_key_or_panic(after) + 1;
        self.insert_pair_at(index, key, value)
    }

    pub fn insert_many_after<I>(&self, after: &K, iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut last_key = after.clone();
        let mut map = self.clone();

        for (key, value) in iter {
            map = map.insert_after(&last_key, value, key.clone());
            last_key = key;
        }

        map
    }

    pub fn insert_before(&self, before: &K, value: V, key: K) -> Self {
        let index = self.index_of_key_or_panic(before);
        self.insert_pair_at(index, key, value)
    }

    pub fn get_between(&self, key1: &K, key2: &K, include_start: bool, include_end: bool) -> Self {
        let pos1 = self.index_of_key_or_panic(key1);
        let pos2 = self.index_of_key_or_panic(key2);

        if pos1 == pos2 {
            if include_start {
                return self.from_selected_order(vec![self.order[pos1].clone()]);
            }

            return Self::new();
        }

        let (left, right) = if pos1 <= pos2 {
            (pos1, pos2)
        } else {
            (pos2, pos1)
        };
        let start = left + usize::from(!include_start);
        let end_exclusive = right + usize::from(include_end);

        if start >= end_exclusive {
            return Self::new();
        }

        let selected = self.order[start..end_exclusive].to_vec();
        self.from_selected_order(selected)
    }

    pub fn get_after(&self, key: &K) -> Option<&V> {
        let index = self.index_of_key_or_panic(key);
        self.order
            .get(index + 1)
            .and_then(|next_key| self.entries.get(next_key))
            .map(|value| value.as_ref())
    }

    pub fn get_before(&self, key: &K) -> Option<&V> {
        let index = self.index_of_key_or_panic(key);
        index
            .checked_sub(1)
            .and_then(|prev_index| self.order.get(prev_index))
            .and_then(|prev_key| self.entries.get(prev_key))
            .map(|value| value.as_ref())
    }

    pub fn reverse(&self) -> Self {
        let mut next = self.clone();
        next.order.reverse();
        next
    }

    pub fn delete_between(&self, key1: &K, key2: &K, delete_start: bool, delete_end: bool) -> Self {
        let pos1 = self.index_of_key_or_panic(key1);
        let pos2 = self.index_of_key_or_panic(key2);

        if pos1 == pos2 {
            if delete_start {
                let selected = self
                    .order
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != pos1)
                    .map(|(_, key)| key.clone())
                    .collect();
                return self.from_selected_order(selected);
            }

            return self.from_selected_order(self.order.clone());
        }

        let (left, right) = if pos1 <= pos2 {
            (pos1, pos2)
        } else {
            (pos2, pos1)
        };
        let start = if delete_start { left } else { left + 1 };
        let end_exclusive = if delete_end { right + 1 } else { right };

        if start >= end_exclusive {
            return self.from_selected_order(self.order.clone());
        }

        let selected = self
            .order
            .iter()
            .enumerate()
            .filter(|(index, _)| *index < start || *index >= end_exclusive)
            .map(|(_, key)| key.clone())
            .collect();
        self.from_selected_order(selected)
    }

    pub fn clear(&self) -> Self {
        if self.is_empty() {
            return self.clone();
        }

        Self::new()
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn to_vec(&self) -> Vec<(K, V)>
    where
        V: Clone,
    {
        self.order
            .iter()
            .map(|key| {
                let value = self
                    .entries
                    .get(key)
                    .expect("linked map invariant violation: missing value for key");
                (key.clone(), value.as_ref().clone())
            })
            .collect()
    }

    pub fn to_map(&self) -> BTreeMap<K, V>
    where
        V: Clone,
    {
        self.entries
            .iter()
            .map(|(key, value)| (key.clone(), value.as_ref().clone()))
            .collect()
    }

    fn insert_pair_at(&self, index: usize, key: K, value: V) -> Self {
        self.insert_shared_at(index, key, Arc::new(value))
    }

    fn insert_shared_at(&self, index: usize, key: K, value: Arc<V>) -> Self {
        if self.contains_key(&key) {
            panic!("cannot insert duplicate key");
        }

        let mut next = self.clone();
        next.order.insert(index, key.clone());
        next.entries.insert(key.clone(), value);

        if self.order.is_empty() {
            next.current = Some(key);
        }

        next
    }

    fn from_selected_order(&self, selected: Vec<K>) -> Self {
        let mut entries = BTreeMap::new();

        for key in &selected {
            let value = self
                .entries
                .get(key)
                .expect("linked map invariant violation: missing value for key");
            entries.insert(key.clone(), value.clone());
        }

        Self {
            entries,
            current: selected.first().cloned(),
            order: selected,
        }
    }
}

impl<K, V> PartialEq for LinkedMap<K, V>
where
    K: Ord + PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order && self.entries == other.entries
    }
}

impl<K, V> Eq for LinkedMap<K, V>
where
    K: Ord + Eq,
    V: Eq,
{
}

impl<K, V> fmt::Display for LinkedMap<K, V>
where
    K: fmt::Debug + Ord,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LinkedMap [")?;

        for (index, key) in self.order.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }

            let value = self
                .entries
                .get(key)
                .expect("linked map invariant violation: missing value for key");
            write!(f, "({:?}, {:?})", key, value)?;
        }

        write!(f, "]")
    }
}

impl<K, V> FromIterator<(K, V)> for LinkedMap<K, V>
where
    K: Clone + Ord,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::from_entries(iter)
    }
}

pub struct Iter<'a, K, V> {
    map: &'a LinkedMap<K, V>,
    front: usize,
    back: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Ord,
{
    type Item = (&'a K, &'a V);

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back.saturating_sub(self.front);
        (remaining, Some(remaining))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }

        let index = self.front;
        self.front += 1;
        let key = self.map.order.get(index)?;
        let value = self.map.entries.get(key)?;
        Some((key, value.as_ref()))
    }
}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V>
where
    K: Ord,
{
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.front >= self.back {
            return None;
        }

        self.back -= 1;
        let key = self.map.order.get(self.back)?;
        let value = self.map.entries.get(key)?;
        Some((key, value.as_ref()))
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> where K: Ord {}

pub struct Values<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Values<'a, K, V>
where
    K: Ord,
{
    type Item = &'a V;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }
}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V>
where
    K: Ord,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, value)| value)
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> where K: Ord {}

impl<'a, K, V> IntoIterator for &'a LinkedMap<K, V>
where
    K: Ord,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[macro_export]
macro_rules! linked_map {
    () => {
        $crate::LinkedMap::new()
    };
    ($($key:expr => $value:expr),+ $(,)?) => {
        $crate::LinkedMap::from_entries([$(($key, $value)),+])
    };
}
