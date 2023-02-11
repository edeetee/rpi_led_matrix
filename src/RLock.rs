use std::sync::{Arc, RwLock, LockResult, RwLockReadGuard};

pub fn split_arwlock<T>(value: T) -> (Arc<RwLock<T>>, RLock<T>) {
    let writer = Arc::new(RwLock::new(value));
    let reader = RLock::new(writer.clone());

    (writer, reader)
}

/// Read only copy of a RWLock
pub struct RLock<T>{
    inner: Arc<RwLock<T>>
}

impl <T> RLock<T> {
    pub fn new(inner: Arc<RwLock<T>>) -> Self {
        Self { inner }
    }

    /// Read the inner content
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.inner.read()
    }
}

impl <T> From<Arc<RwLock<T>>> for RLock<T> {
    fn from(value: Arc<RwLock<T>>) -> Self {
        Self::new(value)
    }
}

// impl <T> AsRef<T> for RLock<T> {
//     fn as_ref(&self) -> &T {
//         self.inner.read()
//     }
// }