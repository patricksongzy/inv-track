use std::collections::HashMap;

use async_graphql::{Error, ErrorExtensions, Result};

use crate::graphql::AppContext;

pub(crate) mod transaction {
    use super::*;

    use async_graphql::CustomValidator;

    use crate::model::item::ItemQuantity;
    use crate::model::transaction::InsertableTransaction;

    pub(crate) struct TransactionQuantityValidator {}

    impl CustomValidator<ItemQuantity> for TransactionQuantityValidator {
        fn check(&self, value: &ItemQuantity) -> Result<(), String> {
            if i32::from(*value) == 0 {
                Err("Transaction cannot have quantity of 0.".to_string())
            } else {
                Ok(())
            }
        }
    }

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
            errors.insert(
                "itemId",
                format!("item with id {:?} not found", transaction.item_id),
            );
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
                errors.insert(
                    "locationId",
                    format!("location with id {:?} not found", transaction.location_id),
                );
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
    use crate::model::item::{InsertableItem, ItemId};

    pub(crate) async fn validate_sku(
        context: &AppContext,
        item: &InsertableItem,
        id: Option<ItemId>,
    ) -> Result<()> {
        if let Some(sku) = &item.sku {
            let id_match = sqlx::query!(
                r#"
                select id from items
                where upper(sku) = upper($1)
                "#,
                sku,
            )
            .fetch_optional(&*context.clients.postgres)
            .await
            .map_err(Error::from)?
            .map(|v| v.id);

            if id_match.is_none() || id.map(i32::from) == id_match {
                Ok(())
            } else {
                Err(Error::new("validation errors on item")
                    .extend_with(|_, e| e.set("itemId", format!("sku {:?} not unique", sku))))
            }
        } else {
            Ok(())
        }
    }
}
