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

impl<T> Collection<T> {
    pub(crate) fn get_active(&self) -> Option<&T> {
        self.active_index
            .map(|active_index| self.items.get(active_index))
            .flatten()
    }

    pub(crate) fn set_active_index(&mut self, index: Option<usize>) {
        self.active_index = index.filter(|index| index < &self.items.len());
    }

    pub(crate) fn find_or_insert_with<P, F>(&mut self, predicate: P, f: F) -> &mut T
    where
        P: FnMut(&mut T) -> bool,
        F: FnOnce() -> T,
    {
        let index = self
            .items
            .iter_mut()
            .position(predicate)
            .unwrap_or_else(|| {
                let index = self.active_index.map(|index| index + 1).unwrap_or_default();
                self.items.insert(index, f());
                index
            });
        self.active_index = Some(index);
        self.items.get_mut(index).unwrap()
    }

    pub(crate) fn remove(&mut self, index: usize) {
        self.items.remove(index);
        if self.active_index == Some(index) {
            let index = index.saturating_sub(1);
            self.active_index = (index < self.items.len()).then_some(index);
        }
    }
}
