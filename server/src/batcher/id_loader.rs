use std::cmp::Eq;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;

use dataloader::non_cached::Loader;
use dataloader::BatchFn;

use crate::batcher;
use crate::error::AppError;

/// A function which retrieves results by ids and constructs a map for them.
/// The keys, `K` are mapped to the values, `T`.
/// The IdMapper takes in some context `C`.
pub(crate) type IdMapper<K, T, C> =
    fn(
        &C,
        Vec<K>,
    ) -> Pin<Box<dyn Future<Output = Result<HashMap<K, T>, AppError>> + Send + '_>>;

/// Handles batched loading of results by ids.
pub(crate) struct IdBatcher<K, T, C> {
    context: C,
    results_by_id: Box<IdMapper<K, T, C>>,
}

/// Batch loader for results by ids.
pub(crate) type IdLoader<K, T, C> = Loader<K, Result<T, AppError>, IdBatcher<K, T, C>>;

#[async_trait]
impl<K, T, C> BatchFn<K, Result<T, AppError>> for IdBatcher<K, T, C>
where
    K: Eq + Hash + Send + Sync + Copy + Clone + Debug,
    T: Send + Clone,
    C: Send + Sync,
{
    async fn load(&mut self, ids: &[K]) -> HashMap<K, Result<T, AppError>> {
        let mut results_map = HashMap::new();
        // get the results by ids
        match (self.results_by_id)(&self.context, ids.to_vec()).await {
            Ok(results) => {
                // add the results to the map
                results_map.extend(results.into_iter().map(|(id, result)| (id, Ok(result))));

                // for each result not found, create an error
                ids.iter().for_each(|id| {
                    if !results_map.contains_key(id) {
                        results_map.insert(
                            *id,
                            Err(AppError::new(
                                "not found".to_string(),
                                juniper::Value::null(),
                            )),
                        );
                    }
                });
            }
            Err(e) => {
                // each request will fail with the error of the batched request
                ids.iter().for_each(|id| {
                    results_map.insert(*id, Err(e.clone()));
                });
            }
        }

        results_map
    }
}

/// Gets an id loader with the given mapping function.
pub(crate) fn get_loader<K, T, C>(context: &C, results_by_id: IdMapper<K, T, C>) -> IdLoader<K, T, C>
where
    K: Eq + Hash + Send + Sync + Copy + Clone + Debug,
    T: Send + Clone,
    C: Send + Sync + Clone,
{
    Loader::new(IdBatcher {
        context: context.clone(),
        results_by_id: Box::new(results_by_id),
    })
    .with_yield_count(batcher::LOADER_YIELD_COUNT)
}

/// Unit tests for the batch loader.
#[cfg(test)]
mod test {
    use super::*;

    /// A fake that adds ids to the context.
    async fn mapper_fake(context: &Option<i32>, ids: Vec<i32>) -> Result<HashMap<i32, i32>, AppError> {
        let mut result = HashMap::new();
        ids.into_iter().for_each(|id| {
            result.insert(id, id + context.unwrap());
        });
        Ok(result)
    }

    /// A fake that returns an error.
    async fn mapper_fail_fake(_: &Option<i32>, _: Vec<i32>) -> Result<HashMap<i32, i32>, AppError> {
        Err(AppError::new("error".to_string(), juniper::Value::null()))
    }

    #[actix_rt::test]
    async fn test_mapper() {
        let context = Some(1);
        let loader = get_loader(&context, |clients, ids| { Box::pin(mapper_fake(clients, ids)) });
        let f1 = loader.load(5);
        let f2 = loader.load(10);
        let f3 = loader.load(1);
        assert_eq!(futures::join!(f1, f2, f3), (Ok(6), Ok(11), Ok(2)));
    }

    #[actix_rt::test]
    async fn test_mapper_fail() {
        let context = Some(1);
        let loader = get_loader(&context, |clients, ids| { Box::pin(mapper_fail_fake(clients, ids)) });
        let f1 = loader.load(5);
        let f2 = loader.load(10);
        let f3 = loader.load(1);
        let e = AppError::new("error".to_string(), juniper::Value::null());
        assert_eq!(futures::join!(f1, f2, f3), (Err(e.clone()), Err(e.clone()), Err(e.clone())));
    }
}
