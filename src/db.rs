use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use sqlx::mysql::{MySqlPool, MySqlRow};
use sqlx::{Row, Column, TypeInfo};
use std::rc::Rc;

// --- MySQL Driver ---

#[php_class]
pub struct AsyncMySql {
    pool: Rc<MySqlPool>,
}

#[php_impl]
impl AsyncMySql {
    pub fn connect(dsn: String, max_conns: i32) -> RustFuture {
        let future = async move {
            // DSN parsing is complex. sqlx::ConnectOptions::from_url handles generic URLs.
            // Expected format: mysql://user:pass@host:port/db
            
            match sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(max_conns as u32)
                .connect(&dsn)
                .await 
            {
                Ok(pool) => {
                    let obj = AsyncMySql { pool: Rc::new(pool) };
                    ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                },
                Err(_e) => {
                    // Return error string or false
                    // For better DX, let's return null and let user check error? 
                    // Or throw exception via wrapper.
                    // Here we return false.
                    let mut z = Zval::new();
                    z.set_bool(false);
                    z
                }
            }
        };
        RustFuture::new(future)
    }

    pub fn query(&self, sql: String) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            match sqlx::query(&sql).fetch_all(pool.as_ref()).await {
                Ok(rows) => {
                    let mut results = ext_php_rs::types::ZendHashTable::new();
                    for row in rows {
                        let row_zval = mysql_row_to_zval(&row);
                        results.push(row_zval).unwrap();
                    }
                    let z = results.into_zval(false).unwrap_or_else(|_| Zval::new());
                    z
                },
                Err(_e) => {
                    // Log error?
                    // eprintln!("Query Error: {}", e);
                    let mut z = Zval::new();
                    z.set_bool(false);
                    z
                }
            }
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            match sqlx::query(&sql).execute(pool.as_ref()).await {
                Ok(done) => {
                    let mut z = Zval::new();
                    z.set_long(done.rows_affected() as i64);
                    z
                },
                Err(_) => {
                    let mut z = Zval::new();
                    z.set_bool(false);
                    z
                }
            }
        };
        RustFuture::new(future)
    }
    
    // Prepare logic is stateful. We need a Statement object.
    // But sqlx is designed to be stateless mostly. 
    // We can simulate prepare/execute by creating a new object AsyncMySqlStatement.
}

// Helper: Convert Row to Assoc Array
fn mysql_row_to_zval(row: &MySqlRow) -> Zval {
    let mut map = ext_php_rs::types::ZendHashTable::new();
    
    for col in row.columns() {
        let name = col.name();
        let type_info = col.type_info();
        let type_name = type_info.name(); // "BOOLEAN", "INT", "VARCHAR", etc.

        // This is a simplified mapping. A full PDO driver needs exhaustive matching.
        let val: Zval = match type_name {
            "BOOLEAN" | "TINYINT" => {
                let v: Option<bool> = row.try_get(name).unwrap_or(None);
                match v {
                    Some(b) => { let mut z = Zval::new(); z.set_bool(b); z },
                    None => Zval::new(),
                }
            },
            "SMALLINT" | "INT" | "INTEGER" | "BIGINT" => {
                let v: Option<i64> = row.try_get(name).unwrap_or(None);
                match v {
                    Some(i) => { let mut z = Zval::new(); z.set_long(i); z },
                    None => Zval::new(),
                }
            },
            "FLOAT" | "DOUBLE" | "DECIMAL" => {
                let v: Option<f64> = row.try_get(name).unwrap_or(None);
                match v {
                    Some(f) => { let mut z = Zval::new(); z.set_double(f); z },
                    None => Zval::new(),
                }
            },
            // Strings, Blobs, Dates, and fallback
            _ => {
                let v: Option<String> = row.try_get(name).unwrap_or(None);
                match v {
                    Some(s) => { let mut z = Zval::new(); z.set_string(&s, false).unwrap(); z },
                    None => Zval::new(),
                }
            }
        };
        
        map.insert(name, val).unwrap();
    }
    
    map.into_zval(false).unwrap_or_else(|_| Zval::new())
}

// --- PostgreSQL Driver ---

use sqlx::postgres::{PgPool, PgRow, PgPoolOptions};

#[php_class]
pub struct AsyncPgSql {
    pool: Rc<PgPool>,
}

#[php_impl]
impl AsyncPgSql {
    pub fn connect(dsn: String, max_conns: i32) -> RustFuture {
        let future = async move {
            match PgPoolOptions::new()
                .max_connections(max_conns as u32)
                .connect(&dsn)
                .await 
            {
                Ok(pool) => {
                    let obj = AsyncPgSql { pool: Rc::new(pool) };
                    ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                },
                Err(_) => {
                    let mut z = Zval::new();
                    z.set_bool(false);
                    z
                }
            }
        };
        RustFuture::new(future)
    }

    pub fn query(&self, sql: String) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            match sqlx::query(&sql).fetch_all(pool.as_ref()).await {
                Ok(rows) => {
                    let mut results = ext_php_rs::types::ZendHashTable::new();
                    for row in rows {
                        let row_zval = pg_row_to_zval(&row);
                        results.push(row_zval).unwrap();
                    }
                    results.into_zval(false).unwrap_or_else(|_| Zval::new())
                },
                Err(_) => {
                    let mut z = Zval::new();
                    z.set_bool(false);
                    z
                }
            }
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            match sqlx::query(&sql).execute(pool.as_ref()).await {
                Ok(done) => {
                    let mut z = Zval::new();
                    z.set_long(done.rows_affected() as i64);
                    z
                },
                Err(_) => {
                    let mut z = Zval::new();
                    z.set_bool(false);
                    z
                }
            }
        };
        RustFuture::new(future)
    }
}

fn pg_row_to_zval(row: &PgRow) -> Zval {
    let mut map = ext_php_rs::types::ZendHashTable::new();
    
    for col in row.columns() {
        let name = col.name();
        let type_info = col.type_info();
        let type_name = type_info.name();

        let val: Zval = match type_name {
            "BOOL" => {
                let v: Option<bool> = row.try_get(name).unwrap_or(None);
                match v {
                    Some(b) => { let mut z = Zval::new(); z.set_bool(b); z },
                    None => Zval::new(),
                }
            },
            "INT2" | "INT4" | "INT8" => {
                let v: Option<i64> = row.try_get(name).unwrap_or(None);
                match v {
                    Some(i) => { let mut z = Zval::new(); z.set_long(i); z },
                    None => Zval::new(),
                }
            },
            "FLOAT4" | "FLOAT8" | "NUMERIC" => {
                let v: Option<f64> = row.try_get(name).unwrap_or(None);
                match v {
                    Some(f) => { let mut z = Zval::new(); z.set_double(f); z },
                    None => Zval::new(),
                }
            },
            _ => {
                let v: Option<String> = row.try_get(name).unwrap_or(None);
                match v {
                    Some(s) => { let mut z = Zval::new(); z.set_string(&s, false).unwrap(); z },
                    None => Zval::new(),
                }
            }
        };
        map.insert(name, val).unwrap();
    }
    map.into_zval(false).unwrap_or_else(|_| Zval::new())
}
