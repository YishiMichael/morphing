use std::slice::Iter;

#[derive(Debug)]
pub(crate) struct Collection<T> {
    items: Vec<T>,
    active_index: Option<usize>,
}

impl<T> Default for Collection<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            active_index: None,
        }
    }
}

impl<T> FromIterator<T> for Collection<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            items: iter.into_iter().collect(),
            active_index: None,
        }
    }
}

pub(crate) trait CollectionItem {
    type Key: PartialEq;

    fn key(&self) -> &Self::Key;
}

impl<T, K> Collection<T>
where
    T: CollectionItem<Key = K>,
    K: PartialEq,
{
    pub(crate) fn get_active(&self) -> Option<&T> {
        self.active_index
            .map(|active_index| self.items.get(active_index))
            .flatten()
    }

    pub(crate) fn set_active(&mut self, key: Option<&K>) {
        self.active_index = key
            .map(|key| self.items.iter().position(|item| item.key() == key))
            .flatten();
    }

    pub(crate) fn iter(&self) -> Iter<'_, T> {
        self.items.iter()
    }

    pub(crate) fn active_find_or_insert_with<F>(&mut self, key: K, f: F) -> &mut T
    where
        F: FnOnce(K) -> T,
    {
        let index = self
            .items
            .iter_mut()
            .position(|item| item.key() == &key)
            .unwrap_or_else(|| {
                let index = self
                    .active_index
                    .map(|index| index + 1)
                    .unwrap_or(self.items.len());
                self.items.insert(index, f(key));
                index
            });
        self.active_index = Some(index);
        self.items.get_mut(index).unwrap()
    }

    pub(crate) fn inactive_find_or_insert_with<F>(&mut self, key: K, f: F) -> &mut T
    where
        F: FnOnce(K) -> T,
    {
        let index = self
            .items
            .iter_mut()
            .position(|item| item.key() == &key)
            .unwrap_or_else(|| {
                let index = self.items.len();
                self.items.insert(index, f(key));
                index
            });
        self.items.get_mut(index).unwrap()
    }

    pub(crate) fn remove(&mut self, key: &K) {
        if let Some(index) = self.items.iter().position(|item| item.key() == key) {
            self.items.remove(index);
            if self.active_index == Some(index) {
                let index = index.saturating_sub(1);
                self.active_index = (index < self.items.len()).then_some(index);
            }
        }
    }
}
