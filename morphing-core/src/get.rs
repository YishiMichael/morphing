#[doc(hidden)]
pub struct Key<const KEY: u64>;

pub trait Get<K> {
    type Output;

    fn get(&self, key: K) -> &Self::Output;
}

impl<const KEY: u64, K, T> Get<(Key<KEY>, K)> for T
where
    T: Get<Key<KEY>>,
    T::Output: Get<K>,
{
    type Output = <T::Output as Get<K>>::Output;

    fn get(&self, key: K) -> &Self::Output {
        self.get(key.0).get(key.1)
    }
}

pub use morphing_macros::key;
