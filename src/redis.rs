use redis::{Client, Commands, Connection, RedisResult};
use serde_derive::Deserialize;
// Define a struct to hold our configuration values
#[derive(Deserialize)]
pub struct Config {
    pub redis: RedisConfig,
}
// Define a struct to hold our Redis configuration values
#[derive(Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub db: u8,
    pub password: Option<String>,
}

// Read config file
fn read_config() -> Config {
    let config_value: String =
        std::fs::read_to_string("Config.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_value).expect("Unable to parse config file");
    config
}

// Define a struct to hold Redis
pub struct RedisInstance {
    pub config: Config,
    pub connection: Connection,
    init: bool,
}

impl Clone for RedisInstance {
    fn clone(&self) -> RedisInstance {
        let instance = RedisInstance::new();
        instance
    }
}

// Implement RedisInstance
impl RedisInstance {
    // new a RedisInstance struct and connect to Redis
    pub fn new() -> RedisInstance {
        let config = read_config();
        let connection_str = format!(
            "redis://{}:{}/{}",
            config.redis.host, config.redis.port, config.redis.db
        );
        let connection = Client::open(connection_str.as_str())
            .expect("Unable to open Redis connection")
            .get_connection()
            .expect("Unable to get Redis connection");
        RedisInstance {
            config,
            connection,
            init: true,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_redis() {
        let mut redis_instance = RedisInstance::new();
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
    }
}
