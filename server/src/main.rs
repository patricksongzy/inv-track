#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]

//! An inventory tracking web application.

#[macro_use]
extern crate juniper;

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

/// Entrypoint for the actix web application.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    let context = Context {
        clients,
        loaders: Arc::new(loaders),
    };

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
