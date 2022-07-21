use fred::{
    interfaces::KeysInterface,
    pool::RedisPool,
    types::{ReconnectPolicy, RedisConfig},
};
use redis::Commands;

async fn get_pool() -> RedisPool {
    let config = RedisConfig::from_url("redis://fhirdbsvr.wiise.azure.net:6379").unwrap();
    let policy = ReconnectPolicy::new_exponential(0, 5000, 30_000, 2);

    let pool = RedisPool::new(config.clone(), 5).unwrap();
    let _ = pool.connect(Some(policy.clone()));
    let _ = pool.wait_for_connect().await.unwrap();

    pool
}

fn get_connection() -> redis::Connection {
    let client = redis::Client::open("redis://fhirdbsvr.wiise.azure.net:6379").unwrap();
    client.get_connection().unwrap()
}

async fn get_using_fred() {
    let conn = get_pool().await;
    match conn
        // THis is the right type of returned value
        .mget::<Vec<Option<String>>, Vec<&str>>(vec![
            "integrationarchivetuftsecw/medicationstatement/files",
            "thiskeydoesnotexist",
        ])
        .await
    {
        Ok(vec) => eprintln!("Just trying to see what the output is with MGET: {:?}", vec),
        Err(e) => eprintln!("Some error from actor: {:?}", e),
    }
}

fn get_using_redis() {
    let mut conn = get_connection();
    match conn.get::<Vec<&str>, Vec<Option<String>>>(vec![
        "integrationarchivetuftsecw/medicationstatement/files",
        "thiskeydoesnotexist",
    ]) {
        Ok(vec) => eprintln!("Just trying to see what the output is with MGET: {:?}", vec),
        Err(e) => eprintln!("Some error from actor: {:?}", e),
    }
}

pub async fn redis_driver() {
    get_using_fred().await;
    get_using_redis();
}
