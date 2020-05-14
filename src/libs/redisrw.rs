extern crate redis;

use redis::{Client, Commands};
use crate::model::message::Message;

#[derive(Clone)]
pub struct redis_connection
{
    client: Client
}

impl redis_connection
{
    pub fn new(conn_str: String) -> redis_connection
    {
        redis_connection
            {
                client: Client::open(format!("redis://{}/", conn_str)).unwrap()
            }
    }

    pub fn write(&self, key: String,  value: String) -> redis::RedisResult<()>
    {
        let mut con = self.client.get_connection()?;
        let _ : () = con.set(key, value)?;
        Ok(())
    }
}