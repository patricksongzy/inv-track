use std::env;

use redis::RedisError;

/// Gets the Redis client, returning the result of the client, or the Redis error.
pub(crate) async fn get_client() -> Result<redis::Client, RedisError> {
    redis::Client::open(env::var("REDIS_URL").expect("REDIS_URL must be set"))
}
