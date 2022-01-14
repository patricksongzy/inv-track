use std::hash::Hash;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::fmt::Debug;
use std::cmp::Eq;

use dataloader::non_cached::Loader;
use dataloader::BatchFn;

use crate::batcher;
use crate::error::AppError;
use crate::graphql::Clients;

/// A function which retrieves results by ids and constructs a map for them.
pub(crate) type IdMapper<K, T> =
    fn(
        &Clients,
        Vec<K>,
    ) -> Pin<Box<dyn Future<Output = Result<HashMap<K, T>, AppError>> + Send + '_>>;

/// Handles batched loading of results by ids.
pub(crate) struct IdBatcher<K, T> {
    clients: Clients,
    results_by_id: Box<IdMapper<K, T>>,
}

/// Batch loader for results by ids.
pub(crate) type IdLoader<K, T> = Loader<K, Result<T, AppError>, IdBatcher<K, T>>;

#[async_trait]
impl<K, T> BatchFn<K, Result<T, AppError>> for IdBatcher<K, T>
where
    K: Eq + Hash + Send + Sync + Copy + Clone + Debug,
    T: Send + Clone
{
    async fn load(&mut self, ids: &[K]) -> HashMap<K, Result<T, AppError>> {
        let mut results_map = HashMap::new();
        // get the results by ids
        match (self.results_by_id)(&self.clients, ids.to_vec()).await {
            Ok(results) => {
                // add the results to the map
                results_map.extend(results.into_iter().map(|(id, result)| (id, Ok(result))));

                // for each result not found, create an error
                ids.into_iter().for_each(|id| {
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
                ids.into_iter().for_each(|id| {
                    results_map.insert(*id, Err(AppError::from(e.clone())));
                });
            }
        }

        results_map
    }
}

/// Gets an id loader with the given mapping function.
pub(crate) fn get_loader<K, T>(
    clients: &Clients,
    results_by_id: IdMapper<K, T>,
) -> IdLoader<K, T>
where
    K: Eq + Hash + Send + Sync + Copy + Clone + Debug,
    T: Send + Clone
{
    Loader::new(IdBatcher {
        clients: clients.clone(),
        results_by_id: Box::new(results_by_id),
    })
    .with_yield_count(batcher::LOADER_YIELD_COUNT)
}
