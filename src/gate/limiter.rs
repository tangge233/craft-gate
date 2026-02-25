use dashmap::DashMap;
use std::sync::Arc;

pub struct Limiter {
    inner: Arc<LimiterInner>,
}

struct LimiterInner {
    counter: DashMap<String, usize>,
    max_count: usize,
}

impl Limiter {
    pub fn new(limit: usize) -> Self {
        Self {
            inner: Arc::new(LimiterInner {
                counter: DashMap::new(),
                max_count: limit,
            }),
        }
    }

    pub fn is_limit_enabled(&self) -> bool {
        self.inner.max_count > 0
    }

    /// 尝试获取许可，成功返回 `Guard`，失败返回 `None`
    pub fn try_acquire(&self, key: impl Into<String>) -> Option<LimiterGuard> {
        let key = key.into();
        let entry = self.inner.counter.entry(key.clone());
        match entry {
            dashmap::Entry::Occupied(mut occ) => {
                let current = *occ.get();
                if current < self.inner.max_count {
                    *occ.get_mut() = current + 1;
                } else {
                    tracing::debug!("Limit reached for {key}");
                    return None;
                }
            }
            dashmap::Entry::Vacant(vac) => {
                vac.insert(1);
            }
        }

        tracing::debug!("Limit increment for {key}");
        Some(LimiterGuard {
            limiter: self.inner.clone(),
            key,
        })
    }
}

pub struct LimiterGuard {
    limiter: Arc<LimiterInner>,
    key: String,
}

impl Drop for LimiterGuard {
    fn drop(&mut self) {
        if let Some(mut entry) = self.limiter.counter.get_mut(&self.key) {
            tracing::debug!("Limit decreament for {0}", &self.key);
            if *entry > 1 {
                *entry -= 1;
            } else {
                drop(entry);
                self.limiter.counter.remove(&self.key);
            }
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_limiter() {
        const TEST_LIMIT: usize = 8;

        let limiter = super::Limiter::new(TEST_LIMIT);

        let mut cache = Vec::with_capacity(TEST_LIMIT);
        for _ in 0..TEST_LIMIT {
            let acquire = limiter.try_acquire("1");
            assert!(acquire.is_some());
            cache.push(acquire.unwrap());
        }

        assert!(limiter.try_acquire("1").is_none());

        for item in cache {
            drop(item);
        }

        assert!(limiter.try_acquire("1").is_some());
    }
}
