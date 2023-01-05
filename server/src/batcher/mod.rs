pub(crate) mod id_loader;

use crate::graphql::Clients;
use crate::model::{item, location, transaction};

/// The default loader yield count.
pub(crate) const LOADER_YIELD_COUNT: usize = 100;

/// Registers the batching data loaders for each retrieval type.
pub(crate) fn register_loaders(
    clients: &Clients,
    loaders: &mut anymap2::Map<dyn anymap2::any::Any + Send + Sync>,
) {
    // get an item by id
    loaders.insert(id_loader::get_loader(clients, |clients, ids| {
        Box::pin(item::get_items_by_ids(clients, ids))
    }));
    // get a location by id
    loaders.insert(id_loader::get_loader(clients, |clients, ids| {
        Box::pin(location::get_locations_by_ids(clients, ids))
    }));
    // get a transaction by id
    loaders.insert(id_loader::get_loader(clients, |clients, ids| {
        Box::pin(transaction::get_transactions_by_ids(clients, ids))
    }));

    // get all transactions for an item
    loaders.insert(id_loader::get_loader(clients, |clients, ids| {
        Box::pin(item::get_transactions_by_item_ids(clients, ids))
    }));
    // get an item quantity
    loaders.insert(id_loader::get_loader(clients, |clients, ids| {
        Box::pin(item::get_quantities_by_item_ids(clients, ids))
    }));
    // get all transactions at a location
    loaders.insert(id_loader::get_loader(clients, |clients, ids| {
        Box::pin(location::get_transactions_by_location_ids(clients, ids))
    }));
}
