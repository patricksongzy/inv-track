use std::collections::HashMap;
use std::fmt::Debug;

use async_graphql::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::batcher::id_loader::IdLoader;
use crate::graphql::{AppContext, Clients};
use crate::model::modification::{self, ModificationType};
use crate::model::transaction::Transaction;

/// The id of a location.
#[derive(PartialEq, Eq, Into, Hash, Copy, Clone, Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub(crate) struct LocationId(i32);
async_graphql::scalar!(LocationId);

/// Location model returned by a query in the inventory tracking system.
#[derive(
    Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize, async_graphql::SimpleObject,
)]
#[graphql(complex)]
pub(crate) struct Location {
    id: LocationId,
    name: String,
    address: Option<String>,
}

/// Location model to input to the inventory tracking system.
#[derive(Debug, PartialEq, Deserialize, async_graphql::InputObject)]
pub(crate) struct InsertableLocation {
    #[graphql(validator(min_length = 1))]
    name: String,
    #[graphql(validator(min_length = 1))]
    address: Option<String>,
}

/// Gets all locations, returning the result, or an error.
pub(crate) async fn get_locations(context: &AppContext) -> Result<Vec<Location>> {
    sqlx::query_as::<_, Location>(
        r#"
        select id, name, address from locations
        order by name
    "#,
    )
    .fetch_all(&*context.clients.postgres)
    .await
    .map_err(Error::from)
}

/// Gets all locations with the given ids.
pub(crate) async fn get_locations_by_ids(
    clients: &Clients,
    ids: Vec<LocationId>,
) -> Result<HashMap<LocationId, Location>> {
    sqlx::query_as::<_, Location>(
        r#"
        select id, name, address from locations
        where id = any($1)
    "#,
    )
    .bind(ids.into_iter().map(|id| id.0).collect::<Vec<i32>>())
    .fetch_all(&*clients.postgres)
    .await
    .map(|locations| {
        locations
            .into_iter()
            .map(|location| (location.id, location))
            .collect()
    })
    .map_err(Error::from)
}

/// Gets all transactions with the given location ids.
pub(crate) async fn get_transactions_by_location_ids(
    clients: &Clients,
    ids: Vec<LocationId>,
) -> Result<HashMap<LocationId, Vec<Transaction>>> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, transaction_date, quantity, comment from transactions
        where location_id = any($1)
        order by transaction_date desc
    "#,
    )
    .bind(ids.into_iter().map(|id| id.0).collect::<Vec<i32>>())
    .fetch_all(&*clients.postgres)
    .await
    .map(|transactions| {
        let mut transactions_map = HashMap::new();
        transactions.into_iter().for_each(|transaction| {
            transactions_map
                .entry(transaction.location_id.unwrap())
                .or_insert(Vec::new())
                .push(transaction);
        });
        transactions_map
    })
    .map_err(Error::from)
}

/// Gets an location, given an id, returning the result, or an error.
pub(crate) async fn get_location(context: &AppContext, id: LocationId) -> Result<Location> {
    context
        .loaders
        .get::<IdLoader<LocationId, Location, Clients>>()
        .unwrap()
        .load(id)
        .await
}

/// Creates an location, given an insertable location, returning the result, or an error.
pub(crate) async fn create_location(
    context: &AppContext,
    location: InsertableLocation,
) -> Result<Location> {
    let created = sqlx::query_as::<_, Location>(
        r#"
        insert into locations (name, address)
        values ($1, $2)
        returning id, name, address
    "#,
    )
    .bind(location.name)
    .bind(location.address)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(Error::from)?;

    // publish the created event using redis pubsub and send the created location data
    modification::broadcast(context, "locations", ModificationType::Create, &created).await;

    Ok(created)
}

/// Updates an location, given an insertable location, returning the result, or an error.
pub(crate) async fn update_location(
    context: &AppContext,
    id: LocationId,
    location: InsertableLocation,
) -> Result<Location> {
    let updated = sqlx::query_as::<_, Location>(
        r#"
        update locations
        set name = $1, address = $2
        where id = $3
        returning id, name, address
    "#,
    )
    .bind(location.name)
    .bind(location.address)
    .bind(id)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(Error::from)?;

    // publish the updated event using redis pubsub and send the created location data
    modification::broadcast(context, "locations", ModificationType::Update, &updated).await;

    Ok(updated)
}

/// Deletes an location, given an id, returning the result, or an error.
pub(crate) async fn delete_location(context: &AppContext, id: LocationId) -> Result<Location> {
    let deleted = sqlx::query_as::<_, Location>(
        r#"
        delete from locations
        where id = $1
        returning id, name, address
    "#,
    )
    .bind(id)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(Error::from)?;

    // publish the deleted event using redis pubsub and send the location data
    modification::broadcast(context, "locations", ModificationType::Delete, &deleted).await;

    Ok(deleted)
}

/// An location in the inventory tracking system.
#[async_graphql::ComplexObject]
impl Location {
    /// The transactions at the location.
    async fn transactions(&self, context: &async_graphql::Context<'_>) -> Vec<Transaction> {
        context
            .data_unchecked::<AppContext>()
            .loaders
            .get::<IdLoader<LocationId, Vec<Transaction>, Clients>>()
            .unwrap()
            .load(self.id)
            .await
            .unwrap_or_default()
    }
}
