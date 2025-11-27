use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use crate::future::RustFuture;
use crate::util::Shared;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult, Value as RedisValue};

/// Async Redis client
#[php_class]
#[php(name = "Async\\Kernel\\Redis\\Client")]
pub struct AsyncRedisClient {
    manager: Shared<Option<ConnectionManager>>,
    last_error: Shared<String>,
}

unsafe impl Send for AsyncRedisClient {}
unsafe impl Sync for AsyncRedisClient {}

#[php_impl]
impl AsyncRedisClient {
    /// Create a new Redis client
    ///
    /// # Example
    /// ```php
    /// $redis = new Redis\Client();
    /// ```
    #[php]
    pub fn __construct() -> Self {
        Self {
            manager: Shared::new(None),
            last_error: Shared::new(String::new()),
        }
    }

    /// Connect to Redis server
    ///
    /// # Parameters
    /// - `host`: Redis server host (default: "127.0.0.1")
    /// - `port`: Redis server port (default: 6379)
    /// - `timeout`: Connection timeout in seconds (default: 0.0, no timeout)
    /// - `reserved`: Reserved parameter (unused, for compatibility)
    /// - `retry_interval`: Retry interval in milliseconds (unused, for compatibility)
    /// - `read_timeout`: Read timeout in seconds (default: 0.0, no timeout)
    ///
    /// # Example
    /// ```php
    /// $redis->connect('127.0.0.1', 6379);
    /// ```
    #[php]
    pub fn connect(
        &mut self,
        host: Option<String>,
        port: Option<i64>,
        timeout: Option<f64>,
        _reserved: Option<&Zval>,
        _retry_interval: Option<i64>,
        read_timeout: Option<f64>,
    ) -> RustFuture {
        let host = host.unwrap_or_else(|| "127.0.0.1".to_string());
        let port = port.unwrap_or(6379);
        let _timeout = timeout.unwrap_or(0.0);
        let _read_timeout = read_timeout.unwrap_or(0.0);

        let manager_ref = self.manager.clone();
        let error_ref = self.last_error.clone();

        let future = async move {
            let url = format!("redis://{}:{}", host, port);

            match redis::Client::open(url) {
                Ok(client) => {
                    match client.get_connection_manager().await {
                        Ok(manager) => {
                            *manager_ref.get_mut() = Some(manager);
                            *error_ref.get_mut() = String::new();

                            let mut z = Zval::new();
                            z.set_bool(true);
                            Ok::<Zval, String>(z)
                        }
                        Err(e) => {
                            *error_ref.get_mut() = e.to_string();
                            Err(format!("Failed to connect: {}", e))
                        }
                    }
                }
                Err(e) => {
                    *error_ref.get_mut() = e.to_string();
                    Err(format!("Invalid Redis URL: {}", e))
                }
            }
        };

        RustFuture::new(future)
    }

    /// Close the Redis connection
    #[php]
    pub fn close(&mut self) -> bool {
        *self.manager.get_mut() = None;
        true
    }

    /// Ping the server
    ///
    /// # Example
    /// ```php
    /// $redis->ping(); // Returns "+PONG"
    /// ```
    #[php]
    pub fn ping(&self, message: Option<String>) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let result: RedisResult<String> = if let Some(msg) = message {
                    redis::cmd("PING").arg(&msg).query_async(&mut conn).await
                } else {
                    redis::cmd("PING").query_async(&mut conn).await
                };

                match result {
                    Ok(response) => Ok(RedisValue::Data(response.into_bytes())),
                    Err(e) => Err(e),
                }
            })
        })
    }

    /// Set string value
    ///
    /// # Example
    /// ```php
    /// $redis->set('key', 'value');
    /// $redis->set('key', 'value', 60); // With 60 seconds expiration
    /// ```
    #[php]
    pub fn set(&self, key: String, value: String, timeout: Option<i64>) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let result: RedisResult<String> = if let Some(seconds) = timeout {
                    conn.set_ex(&key, &value, seconds as u64).await
                } else {
                    conn.set(&key, &value).await
                };

                match result {
                    Ok(_) => Ok(RedisValue::Data(b"OK".to_vec())),
                    Err(e) => Err(e),
                }
            })
        })
    }

    /// Get string value
    ///
    /// # Example
    /// ```php
    /// $value = $redis->get('key');
    /// ```
    #[php]
    pub fn get(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                conn.get::<_, Option<Vec<u8>>>(&key).await.map(|opt| {
                    opt.map(RedisValue::Data).unwrap_or(RedisValue::Nil)
                })
            })
        })
    }

    /// Delete one or more keys
    ///
    /// # Example
    /// ```php
    /// $redis->del('key1');
    /// $redis->del(['key1', 'key2', 'key3']);
    /// ```
    #[php]
    pub fn del(&self, keys: &Zval) -> RustFuture {
        let keys_vec = if let Some(arr) = keys.array() {
            arr.values()
                .filter_map(|v| v.string().map(String::from))
                .collect::<Vec<_>>()
        } else if let Some(s) = keys.string() {
            vec![s]
        } else {
            vec![]
        };

        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let count: i64 = redis::cmd("DEL")
                    .arg(&keys_vec)
                    .query_async(&mut conn)
                    .await?;
                Ok(RedisValue::Int(count))
            })
        })
    }

    /// Check if key exists
    ///
    /// # Example
    /// ```php
    /// $exists = $redis->exists('key'); // Returns 1 or 0
    /// ```
    #[php]
    pub fn exists(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let exists: bool = conn.exists(&key).await?;
                Ok(RedisValue::Int(if exists { 1 } else { 0 }))
            })
        })
    }

    /// Set expiration on key
    ///
    /// # Example
    /// ```php
    /// $redis->expire('key', 60); // Expire in 60 seconds
    /// ```
    #[php]
    pub fn expire(&self, key: String, seconds: i64) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let result: bool = conn.expire(&key, seconds).await?;
                Ok(RedisValue::Int(if result { 1 } else { 0 }))
            })
        })
    }

    /// Get time to live for key
    ///
    /// # Example
    /// ```php
    /// $ttl = $redis->ttl('key'); // Returns seconds, -1 if no expiry, -2 if key doesn't exist
    /// ```
    #[php]
    pub fn ttl(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let ttl: i64 = conn.ttl(&key).await?;
                Ok(RedisValue::Int(ttl))
            })
        })
    }

    /// Increment value
    ///
    /// # Example
    /// ```php
    /// $redis->incr('counter'); // Returns new value
    /// ```
    #[php]
    pub fn incr(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let value: i64 = conn.incr(&key, 1).await?;
                Ok(RedisValue::Int(value))
            })
        })
    }

    /// Increment by value
    ///
    /// # Example
    /// ```php
    /// $redis->incrBy('counter', 5); // Increment by 5
    /// ```
    #[php]
    pub fn incr_by(&self, key: String, value: i64) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let new_value: i64 = conn.incr(&key, value).await?;
                Ok(RedisValue::Int(new_value))
            })
        })
    }

    /// Decrement value
    ///
    /// # Example
    /// ```php
    /// $redis->decr('counter'); // Returns new value
    /// ```
    #[php]
    pub fn decr(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let value: i64 = conn.decr(&key, 1).await?;
                Ok(RedisValue::Int(value))
            })
        })
    }

    /// Decrement by value
    ///
    /// # Example
    /// ```php
    /// $redis->decrBy('counter', 3); // Decrement by 3
    /// ```
    #[php]
    pub fn decr_by(&self, key: String, value: i64) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let new_value: i64 = conn.decr(&key, value).await?;
                Ok(RedisValue::Int(new_value))
            })
        })
    }

    /// Push value to list (left)
    ///
    /// # Example
    /// ```php
    /// $redis->lPush('list', 'value1', 'value2');
    /// ```
    #[php]
    pub fn l_push(&self, key: String, values: Vec<String>) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let len: i64 = redis::cmd("LPUSH")
                    .arg(&key)
                    .arg(&values)
                    .query_async(&mut conn)
                    .await?;
                Ok(RedisValue::Int(len))
            })
        })
    }

    /// Push value to list (right)
    ///
    /// # Example
    /// ```php
    /// $redis->rPush('list', 'value1', 'value2');
    /// ```
    #[php]
    pub fn r_push(&self, key: String, values: Vec<String>) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let len: i64 = redis::cmd("RPUSH")
                    .arg(&key)
                    .arg(&values)
                    .query_async(&mut conn)
                    .await?;
                Ok(RedisValue::Int(len))
            })
        })
    }

    /// Pop value from list (left)
    ///
    /// # Example
    /// ```php
    /// $value = $redis->lPop('list');
    /// ```
    #[php]
    pub fn l_pop(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                conn.lpop::<_, Option<Vec<u8>>>(&key, None).await.map(|opt| {
                    opt.map(RedisValue::Data).unwrap_or(RedisValue::Nil)
                })
            })
        })
    }

    /// Pop value from list (right)
    ///
    /// # Example
    /// ```php
    /// $value = $redis->rPop('list');
    /// ```
    #[php]
    pub fn r_pop(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                conn.rpop::<_, Option<Vec<u8>>>(&key, None).await.map(|opt| {
                    opt.map(RedisValue::Data).unwrap_or(RedisValue::Nil)
                })
            })
        })
    }

    /// Get list length
    ///
    /// # Example
    /// ```php
    /// $length = $redis->lLen('list');
    /// ```
    #[php]
    pub fn l_len(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let len: i64 = conn.llen(&key).await?;
                Ok(RedisValue::Int(len))
            })
        })
    }

    /// Get list range
    ///
    /// # Example
    /// ```php
    /// $values = $redis->lRange('list', 0, -1); // Get all elements
    /// ```
    #[php]
    pub fn l_range(&self, key: String, start: i64, stop: i64) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let values: Vec<Vec<u8>> = conn.lrange(&key, start as isize, stop as isize).await?;
                Ok(RedisValue::Bulk(
                    values.into_iter().map(RedisValue::Data).collect()
                ))
            })
        })
    }

    /// Set hash field
    ///
    /// # Example
    /// ```php
    /// $redis->hSet('user:1', 'name', 'John');
    /// ```
    #[php]
    pub fn h_set(&self, key: String, field: String, value: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let result: bool = conn.hset(&key, &field, &value).await?;
                Ok(RedisValue::Int(if result { 1 } else { 0 }))
            })
        })
    }

    /// Get hash field
    ///
    /// # Example
    /// ```php
    /// $name = $redis->hGet('user:1', 'name');
    /// ```
    #[php]
    pub fn h_get(&self, key: String, field: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                conn.hget::<_, _, Option<Vec<u8>>>(&key, &field).await.map(|opt| {
                    opt.map(RedisValue::Data).unwrap_or(RedisValue::Nil)
                })
            })
        })
    }

    /// Get all hash fields and values
    ///
    /// # Example
    /// ```php
    /// $user = $redis->hGetAll('user:1');
    /// ```
    #[php]
    pub fn h_get_all(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let result: Vec<(Vec<u8>, Vec<u8>)> = conn.hgetall(&key).await?;
                let map: Vec<RedisValue> = result
                    .into_iter()
                    .flat_map(|(k, v)| vec![RedisValue::Data(k), RedisValue::Data(v)])
                    .collect();
                Ok(RedisValue::Bulk(map))
            })
        })
    }

    /// Delete hash field
    ///
    /// # Example
    /// ```php
    /// $redis->hDel('user:1', 'name');
    /// ```
    #[php]
    pub fn h_del(&self, key: String, fields: Vec<String>) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let count: i64 = redis::cmd("HDEL")
                    .arg(&key)
                    .arg(&fields)
                    .query_async(&mut conn)
                    .await?;
                Ok(RedisValue::Int(count))
            })
        })
    }

    /// Check if hash field exists
    ///
    /// # Example
    /// ```php
    /// $exists = $redis->hExists('user:1', 'name');
    /// ```
    #[php]
    pub fn h_exists(&self, key: String, field: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let exists: bool = conn.hexists(&key, &field).await?;
                Ok(RedisValue::Int(if exists { 1 } else { 0 }))
            })
        })
    }

    /// Add member to set
    ///
    /// # Example
    /// ```php
    /// $redis->sAdd('tags', 'php', 'rust', 'async');
    /// ```
    #[php]
    pub fn s_add(&self, key: String, members: Vec<String>) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let count: i64 = redis::cmd("SADD")
                    .arg(&key)
                    .arg(&members)
                    .query_async(&mut conn)
                    .await?;
                Ok(RedisValue::Int(count))
            })
        })
    }

    /// Get all set members
    ///
    /// # Example
    /// ```php
    /// $tags = $redis->sMembers('tags');
    /// ```
    #[php]
    pub fn s_members(&self, key: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let members: Vec<Vec<u8>> = conn.smembers(&key).await?;
                Ok(RedisValue::Bulk(
                    members.into_iter().map(RedisValue::Data).collect()
                ))
            })
        })
    }

    /// Check if member exists in set
    ///
    /// # Example
    /// ```php
    /// $isMember = $redis->sIsMember('tags', 'php');
    /// ```
    #[php]
    pub fn s_is_member(&self, key: String, member: String) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let is_member: bool = conn.sismember(&key, &member).await?;
                Ok(RedisValue::Int(if is_member { 1 } else { 0 }))
            })
        })
    }

    /// Remove member from set
    ///
    /// # Example
    /// ```php
    /// $redis->sRem('tags', 'php');
    /// ```
    #[php]
    pub fn s_rem(&self, key: String, members: Vec<String>) -> RustFuture {
        self.execute_command(move |mut conn| {
            Box::pin(async move {
                let count: i64 = redis::cmd("SREM")
                    .arg(&key)
                    .arg(&members)
                    .query_async(&mut conn)
                    .await?;
                Ok(RedisValue::Int(count))
            })
        })
    }

    /// Get last error message
    #[php]
    pub fn get_last_error(&self) -> String {
        self.last_error.get_ref().clone()
    }

    /// Clear last error
    #[php]
    pub fn clear_last_error(&mut self) {
        *self.last_error.get_mut() = String::new();
    }
}

// Helper methods (not exposed to PHP)
impl AsyncRedisClient {
    /// Helper to execute commands
    fn execute_command<F, Fut>(&self, f: F) -> RustFuture
    where
        F: FnOnce(ConnectionManager) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = RedisResult<RedisValue>> + Send + 'static,
    {
        let manager_opt = self.manager.get_ref().clone();
        let error_ref = self.last_error.clone();

        let future = async move {
            let manager = manager_opt
                .ok_or_else(|| "Not connected to Redis".to_string())?;

            match f(manager).await {
                Ok(value) => {
                    *error_ref.get_mut() = String::new();
                    redis_value_to_zval(&value)
                }
                Err(e) => {
                    *error_ref.get_mut() = e.to_string();
                    Err(format!("Redis error: {}", e))
                }
            }
        };

        RustFuture::new(future)
    }
}

/// Convert Redis value to Zval
fn redis_value_to_zval(value: &RedisValue) -> Result<Zval, String> {
    match value {
        RedisValue::Nil => Ok(Zval::new()), // null
        RedisValue::Int(i) => {
            let mut z = Zval::new();
            z.set_long(*i);
            Ok(z)
        }
        RedisValue::Data(bytes) => {
            let mut z = Zval::new();
            // Try to convert to string if valid UTF-8, otherwise return as binary
            match String::from_utf8(bytes.clone()) {
                Ok(s) => {
                    z.set_string(&s, false)
                        .map_err(|e| format!("Failed to set string: {:?}", e))?;
                }
                Err(_) => {
                    z.set_binary(bytes.clone());
                }
            }
            Ok(z)
        }
        RedisValue::Bulk(values) => {
            let mut arr = ext_php_rs::types::ZendHashTable::new();
            for (i, val) in values.iter().enumerate() {
                let z = redis_value_to_zval(val)?;
                arr.insert_at_index(i as i64, z)
                    .map_err(|e| format!("Failed to insert into array: {:?}", e))?;
            }
            let mut z = Zval::new();
            z.set_hashtable(arr);
            Ok(z)
        }
        RedisValue::Status(s) => {
            let mut z = Zval::new();
            z.set_string(s, false)
                .map_err(|e| format!("Failed to set string: {:?}", e))?;
            Ok(z)
        }
        RedisValue::Okay => {
            let mut z = Zval::new();
            z.set_bool(true);
            Ok(z)
        }
    }
}
