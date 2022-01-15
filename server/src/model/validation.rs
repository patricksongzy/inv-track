use std::collections::HashMap;

use async_graphql::{Error, ErrorExtensions, Result};

use crate::graphql::AppContext;

pub(crate) mod transaction {
    use super::*;
    use crate::model::transaction::InsertableTransaction;

    /// Validates that the item and location for a transaction exist.
    pub(crate) async fn validate_ids(
        context: &AppContext,
        transaction: &InsertableTransaction,
    ) -> Result<()> {
        let mut errors = HashMap::new();

        // check item exists
        let item_count = sqlx::query!(
            r#"select count(id) from items where id = $1"#,
            i32::from(transaction.item_id)
        )
        .fetch_one(&*context.clients.postgres)
        .await
        .map_err(Error::from)?
        .count
        .unwrap_or(0);

        if item_count != 1 {
            errors.insert("itemId", format!("item with id {:?} not found", transaction.item_id));
        }

        // check location exists
        if let Some(location_id) = transaction.location_id {
            let location_count = sqlx::query!(
                r#"select count(id) from locations where id = $1"#,
                i32::from(location_id)
            )
            .fetch_one(&*context.clients.postgres)
            .await
            .map_err(Error::from)?
            .count
            .unwrap_or(0);

            if location_count != 1 {
                errors.insert("locationId", format!("location with id {:?} not found", transaction.location_id));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let mut error = Error::new("validation errors on transaction");
            for (k, v) in errors {
                error = error.extend_with(|_, e| e.set(k, v));
            }
            Err(error)
        }
    }
}

pub(crate) mod item {
    use super::*;
    use crate::model::item::InsertableItem;

    pub(crate) async fn validate_sku(
        context: &AppContext,
        item: &InsertableItem,
    ) -> Result<()> {
        if let Some(sku) = &item.sku {
            let sku_count = sqlx::query!(
                r#"select count(id) from items where upper(sku) = upper($1)"#,
                sku
            )
            .fetch_one(&*context.clients.postgres)
            .await
            .map_err(Error::from)?
            .count
            .unwrap_or(0);

            if sku_count == 0 {
                Ok(())
            } else {
                Err(Error::new("validation errors on item").extend_with(|_, e| e.set("itemId", format!("sku {:?} not unique", sku))))
            }
        } else {
            Ok(())
        }
    }
}
