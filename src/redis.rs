use redis::{Client, Commands, Connection, RedisResult};
use serde_derive::Deserialize;

use crate::get_profile;
use crate::init::{get_redis_config, Config};


// Define a struct to hold Redis
pub struct RedisInstance {
    pub connection: Connection,
    init: bool,
    pub profile: String,
}

impl Clone for RedisInstance {
    fn clone(&self) -> RedisInstance {
        RedisInstance::new(self.profile.as_str())
    }
}

// Implement RedisInstance
impl RedisInstance {
    // new a RedisInstance struct and connect to Redis
    pub fn new(profile: &str) -> RedisInstance {
        let redis_config = get_redis_config(&profile);
        let connection_str = format!(
            "redis://{}:{}/{}",
            redis_config.host, redis_config.port, redis_config.db
        );
        let connection = Client::open(connection_str.as_str())
            .expect("Unable to open Redis connection")
            .get_connection()
            .expect("Unable to get Redis connection");
        RedisInstance {
            connection,
            init: true,
            profile: String::from(profile),
        }
    }

    // check if redis is initialized return a RedisResult
    fn check_init(&self) -> RedisResult<bool> {
        if self.init {
            RedisResult::Ok(true)
        } else {
            Err(redis::RedisError::from((
                redis::ErrorKind::InvalidClientConfig,
                "Redis not initialized",
            )))
        }
    }

    // get a value from Redis
    pub fn get(&mut self, key: &str) -> RedisResult<String> {
        // check if Redis is initialized
        self.check_init()?;
        self.connection.get(key)
    }

    // set a value to Redis
    pub fn set(&mut self, key: &str, value: &str) -> RedisResult<()> {
        self.check_init()?;
        self.connection.set(key, value)
    }

    // set a value with expire to Redis
    pub fn set_with_expire(&mut self, key: &str, value: &str, expire: u64) -> RedisResult<()> {
        self.check_init()?;
        self.connection.set_ex(key, value, expire)
    }

    // set expire to Redis
    pub fn set_expire(&mut self, key: &str, expire: i64) -> RedisResult<()> {
        self.check_init()?;
        self.connection.expire(key, expire)
    }

    // delete a value from Redis
    pub fn delete(&mut self, key: &str) -> RedisResult<()> {
        self.check_init()?;
        self.connection.del(key)
    }

    // check if a key exists in Redis
    pub fn exists(&mut self, key: &str) -> RedisResult<bool> {
        self.check_init()?;
        self.connection.exists(key)
    }

    // set list to Redis
    pub fn set_list(&mut self, key: &str, value: &Vec<String>) -> RedisResult<()> {
        self.check_init()?;
        self.connection.rpush(key, value)
    }

    // set list with expire to Redis
    pub fn set_list_expire(
        &mut self,
        key: &str,
        value: &Vec<String>,
        expire: i64,
    ) -> RedisResult<()> {
        self.check_init()?;
        self.connection.rpush(key, value)?;
        self.connection.expire(key, expire)
    }

    // Push element to list
    pub fn push_list(&mut self, key: &str, value: &str) -> RedisResult<()> {
        self.check_init()?;
        self.connection.rpush(key, value)
    }

    // remove element from list
    pub fn remove_list(&mut self, key: &str, value: &str) -> RedisResult<()> {
        self.check_init()?;
        self.connection.lrem(key, 0, value)
    }

    // get list from Redis
    pub fn get_list(&mut self, key: &str) -> RedisResult<Vec<String>> {
        self.check_init()?;
        self.connection.lrange(key, 0, -1)
    }

    // exist in list 
    pub fn exist_in_list(&mut self, key: &str, value: &str) -> RedisResult<bool> {
        self.check_init()?;
        let list = self.connection.lrange(key, 0, -1);
        let list:Vec<String> = match list {
            Ok(list) => list,
            Err(e) => {
                tracing::error!("Redis execution error: {}", e);
                return RedisResult::Ok(false);
            }
        };
        for item in list.iter() {
            if item == value {
                return RedisResult::Ok(true);
            }
        }
        RedisResult::Ok(false)
    }

    // push hash to redis
    pub fn push_hash_expire(&mut self, key: &str, field: &str, value: &str, expire: i64) -> RedisResult<()> {
        self.check_init()?;
        self.connection.hset(key, field, value)?;
        self.connection.expire(key, expire)
    }

    // get hash from redis
    pub fn get_hash(&mut self, key: &str, field: &str) -> RedisResult<String> {
        self.check_init()?;
        self.connection.hget(key, field)
    }

    // remove hash from redis
    pub fn remove_hash(&mut self, key: &str, field: &str) -> RedisResult<()> {
        self.check_init()?;
        self.connection.hdel(key, field)
    }

    // get all exists keys in specific layer from redis
    pub fn get_all_keys(&mut self, key: &str) -> RedisResult<Vec<String>> {
        self.check_init()?;
        self.connection.keys(key)
    }

    // get all exists keys name in specific layer from redis
    pub fn get_all_keys_name(&mut self, key: &str) -> RedisResult<Vec<String>> {
        self.check_init()?;
        let keys: Vec<String> = self.connection.keys(key)?;
        let mut keys_name: Vec<String> = Vec::new();
        // calculate the number of layers in key
        let key_layer = key.split(":").collect::<Vec<&str>>().len();
        for key in keys.iter() {
            let key = key.split(":").collect::<Vec<&str>>();
            if key.len() < key_layer {
                continue;
            }
            let key = key[key_layer - 1];
            keys_name.push(String::from(key));
        }
        RedisResult::Ok(keys_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_redis() {
        let mut redis_instance = RedisInstance::new("local");
        let key = "test";
        let value = "test";
        // let res = redis_instance.set(key, value);
        // assert_eq!(res.is_ok(), true);
        // let res = redis_instance.get(key);
        // assert_eq!(res.is_ok(), true);
        // assert_eq!(res.unwrap(), value);
        // let res = redis_instance.delete(key);
        // assert_eq!(res.is_ok(), true);
        // let res = redis_instance.get(key);
        // assert_eq!(res.is_err(), true);
        // let res = redis_instance.set_list(key, &vec![String::from("test1"), String::from("test2")]);
        // assert_eq!(res.is_ok(), true);
        let res = redis_instance.set_list_expire(
            key,
            &vec![String::from("test1"), String::from("test2")],
            60,
        );
        assert_eq!(res.is_ok(), true);
        let res = redis_instance.get_list(key);
        println!("{:?}", res.ok());
        let res = redis_instance.get_all_keys("Currency:*");
        println!("{:?}", res.ok());
        let res = redis_instance.get_all_keys_name("Data:*");
        println!("{:?}", res.ok());
    }
}
