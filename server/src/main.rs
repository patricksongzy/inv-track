#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]

//! An inventory tracking web application.

#[macro_use]
extern crate juniper;
#[macro_use]
extern crate derive_more;

mod batcher;
mod db;
mod error;
mod graphql;
mod model;
mod store;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};

use crate::graphql::{Clients, Context};

/// Connection config keep alive interval in seconds.
const CONFIG_KEEP_ALIVE: u64 = 15;

/// The route for the GraphQL playground.
async fn playground_route() -> Result<HttpResponse, Error> {
    juniper_actix::playground_handler("/graphql", Some("/subscriptions")).await
}

/// The route for the GraphQL endpoint.
async fn graphql_route(
    req: HttpRequest,
    payload: web::Payload,
    context: web::Data<Context>,
    schema: web::Data<graphql::Schema>,
) -> Result<HttpResponse, Error> {
    juniper_actix::graphql_handler(&schema, &context, req, payload).await
}

/// The route for the GraphQL subscriptions.
async fn subscriptions_route(
    req: HttpRequest,
    payload: web::Payload,
    context: web::Data<Context>,
    schema: web::Data<graphql::Schema>,
) -> Result<HttpResponse, Error> {
    let config = juniper_graphql_ws::ConnectionConfig::new((*context.into_inner()).clone());
    let config = config.with_keep_alive_interval(Duration::from_secs(CONFIG_KEEP_ALIVE));
    juniper_actix::subscriptions::subscriptions_handler(req, payload, schema.into_inner(), config)
        .await
}

/// Gets the context for the application.
async fn get_context() -> Context {
    // create the redis client and db pool, storing them in the context
    let redis = Arc::new(
        store::get_client()
            .await
            .expect("unable to connect to redis"),
    );
    let postgres = Arc::new(db::get_pool().await);

    let clients = Clients { postgres, redis };

    let mut loaders = anymap::Map::new();
    batcher::register_loaders(&clients, &mut loaders);

    Context {
        clients,
        loaders: Arc::new(loaders),
    }
}

/// Entrypoint for the actix web application.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let context = get_context().await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(context.clone()))
            .app_data(web::Data::new(graphql::schema()))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
            .service(web::resource("/subscriptions").route(web::get().to(subscriptions_route)))
            .service(web::resource("/playground").route(web::get().to(playground_route)))
    })
    .bind(format!(
        "{}:{}",
        env::var("ACTIX_ADDRESS").expect("ACTIX_ADDRESS must be set"),
        env::var("ACTIX_PORT").expect("ACTIX_PORT must be set")
    ))?
    .run()
    .await
}

/// Integration level tests.
#[cfg(test)]
mod test {
    use super::*;

    use actix_web::{http, test};

    /// Macro to set up the test server.
    macro_rules! test_server {
        () => {{
            let context = get_context().await;
            test::init_service(
                App::new()
                    .app_data(web::Data::new(context.clone()))
                    .app_data(web::Data::new(graphql::schema()))
                    .service(web::resource("/graphql").route(web::post().to(graphql_route))),
            )
            .await
        }};
    }

    #[actix_rt::test]
    async fn test_empty_item_name() {
        let mut app = test_server!();
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query": r#"mutation { createItem(item: { name: "" }) { id } }"#
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp["errors"][0]["extensions"][0]["field"], "name");
    }

    #[actix_rt::test]
    async fn test_duplicate_item_sku() {
        let mut app = test_server!();
        let mut resp: serde_json::value::Value = serde_json::json!({});
        for _ in 0..2 {
            let req = test::TestRequest::post()
                .uri("/graphql")
                .insert_header(http::header::ContentType::json())
                .set_json(serde_json::json!({
                    "query": r#"mutation { createItem(item: { name: "name", sku: "ABC" }) { id } }"#
                }))
                .to_request();
            resp = test::call_and_read_body_json(&mut app, req).await;
        }
        assert_eq!(resp["errors"][0]["extensions"][0]["field"], "sku");
    }

    #[actix_rt::test]
    async fn test_nonexistent_transaction_item() {
        let mut app = test_server!();
        let req = test::TestRequest::post().uri("/graphql").insert_header(http::header::ContentType::json()).set_json(serde_json::json!({
            "query": r#"mutation { createTransaction(transaction: { itemId: 0, quantity: 10 }) { id } }"#
        })).to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp["errors"][0]["extensions"][0]["field"], "itemId");
    }

    #[actix_rt::test]
    async fn test_nonexistent_transaction_location() {
        let mut app = test_server!();
        // create a test item
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query": r#"mutation { createItem(item: { name: "TestItem" }) { id } }"#
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        let id = resp["data"]["createItem"]["id"].as_i64().unwrap();

        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"mutation { createTransaction(transaction: { itemId: "#,
                        id,
                        r#", locationId: 0, quantity: 10 }) { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp["errors"][0]["extensions"][0]["field"], "locationId");
    }

    #[actix_rt::test]
    async fn test_delete_location_nulls_and_delete_item_deletes_transaction() {
        let mut app = test_server!();
        // create a test item
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query": r#"mutation { createItem(item: { name: "TestItem" }) { id } }"#
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        let item_id = resp["data"]["createItem"]["id"].as_i64().unwrap();

        // create a test location
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query": r#"mutation { createLocation(location: { name: "Toronto" }) { id } }"#
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        let location_id = resp["data"]["createLocation"]["id"].as_i64().unwrap();

        // create a test transaction
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"mutation { createTransaction(transaction: { itemId: "#,
                        item_id,
                        r#", quantity: 10 }) { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        let transaction_id = resp["data"]["createTransaction"]["id"].as_i64().unwrap();

        // delete the test location
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"mutation { deleteLocation(id: "#, location_id, r#" ) { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(
            resp["data"]["deleteLocation"]["id"].as_i64().unwrap(),
            location_id
        );

        // check that the test transaction location is null
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"{ transaction(id: "#, transaction_id, r#") { location { id } } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(
            resp["data"]["transaction"]["location"],
            serde_json::json!(null)
        );

        // delete the test item
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"mutation { deleteItem(id: "#, item_id, r#" ) { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp["data"]["deleteItem"]["id"].as_i64().unwrap(), item_id);

        // check that the test transaction was deleted
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"{ transaction(id: "#, transaction_id, r#") { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp["errors"][0]["message"].as_str().unwrap(), "not found");
    }
}
