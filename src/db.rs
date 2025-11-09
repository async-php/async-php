use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use sqlx::mysql::{MySqlPool, MySqlRow};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Row, Column, TypeInfo, Transaction, Postgres, MySql};
use crate::util::Shared;

// ======================================================================================
// MySQL Implementation
// ======================================================================================

#[php_class]
#[php(name = "Async\\Kernel\\DB\\MySql")]
pub struct AsyncMySql {
    pool: Shared<MySqlPool>,
}

#[php_impl]
impl AsyncMySql {
    pub fn connect(dsn: String, max_conns: i32) -> RustFuture {
        let future = async move {
            match sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(max_conns as u32)
                .connect(&dsn)
                .await 
            {
                Ok(pool) => {
                    let obj = AsyncMySql { pool: Shared::new(pool) };
                    ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                },
                Err(_e) => Zval::new()
            }
        };
        RustFuture::new(future)
    }

    pub fn query(&self, sql: String, params: Option<Vec<String>>) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            let mut query = sqlx::query(&sql);
            if let Some(args) = params {
                for arg in args {
                    query = query.bind(arg);
                }
            }

            match query.fetch_all(pool.get_ref()).await {
                Ok(rows) => {
                    let mut results = ext_php_rs::types::ZendHashTable::new();
                    for row in rows {
                        results.push(mysql_row_to_zval(&row)).unwrap();
                    }
                    results.into_zval(false).unwrap_or_else(|_| Zval::new())
                },
                Err(_e) => Zval::new()
            }
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String, params: Option<Vec<String>>) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            let mut query = sqlx::query(&sql);
            if let Some(args) = params {
                for arg in args {
                    query = query.bind(arg);
                }
            }

            match query.execute(pool.get_ref()).await {
                Ok(done) => {
                    let mut z = Zval::new();
                    z.set_long(done.rows_affected() as i64);
                    z
                },
                Err(_) => Zval::new()
            }
        };
        RustFuture::new(future)
    }

    pub fn begin_transaction(&self) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            match pool.begin().await {
                Ok(tx) => {
                    let obj = AsyncMySqlTransaction {
                        tx: Shared::new(Some(tx))
                    };
                    ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                },
                Err(_) => Zval::new()
            }
        };
        RustFuture::new(future)
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\DB\\MySql\\Transaction")]
pub struct AsyncMySqlTransaction {
    // Option because commit/rollback consumes the transaction
    tx: Shared<Option<Transaction<'static, MySql>>>,
}

#[php_impl]
impl AsyncMySqlTransaction {
    pub fn query(&self, sql: String, params: Option<Vec<String>>) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            if let Some(tx) = tx_rc.get_mut().as_mut() {
                let mut query = sqlx::query(&sql);
                if let Some(args) = params {
                    for arg in args {
                        query = query.bind(arg);
                    }
                }

                match query.fetch_all(&mut **tx).await {
                    Ok(rows) => {
                        let mut results = ext_php_rs::types::ZendHashTable::new();
                        for row in rows {
                            results.push(mysql_row_to_zval(&row)).unwrap();
                        }
                        results.into_zval(false).unwrap_or_else(|_| Zval::new())
                    },
                    Err(_) => Zval::new()
                }
            } else {
                Zval::new() // Transaction already finished
            }
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String, params: Option<Vec<String>>) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            if let Some(tx) = tx_rc.get_mut().as_mut() {
                let mut query = sqlx::query(&sql);
                if let Some(args) = params {
                    for arg in args {
                        query = query.bind(arg);
                    }
                }

                match query.execute(&mut **tx).await {
                    Ok(done) => {
                        let mut z = Zval::new();
                        z.set_long(done.rows_affected() as i64);
                        z
                    },
                    Err(_) => Zval::new()
                }
            } else {
                Zval::new()
            }
        };
        RustFuture::new(future)
    }

    pub fn commit(&self) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            if let Some(tx) = tx_rc.get_mut().take() {
                let mut z = Zval::new();
                z.set_bool(tx.commit().await.is_ok());
                z
            } else {
                let mut z = Zval::new();
                z.set_bool(false);
                z
            }
        };
        RustFuture::new(future)
    }

    pub fn rollback(&self) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            if let Some(tx) = tx_rc.get_mut().take() {
                let mut z = Zval::new();
                z.set_bool(tx.rollback().await.is_ok());
                z
            } else {
                let mut z = Zval::new();
                z.set_bool(false);
                z
            }
        };
        RustFuture::new(future)
    }
}

fn mysql_row_to_zval(row: &MySqlRow) -> Zval {
    let mut map = ext_php_rs::types::ZendHashTable::new();
    for col in row.columns() {
        let name = col.name();
        let type_name = col.type_info().name();
        let val: Zval = match type_name {
            "BOOLEAN" | "TINYINT" => {
                row.try_get::<Option<bool>, _>(name).unwrap_or(None).map(|v| { let mut z = Zval::new(); z.set_bool(v); z }).unwrap_or_else(Zval::new)
            },
            "SMALLINT" | "INT" | "INTEGER" | "BIGINT" => {
                row.try_get::<Option<i64>, _>(name).unwrap_or(None).map(|v| { let mut z = Zval::new(); z.set_long(v); z }).unwrap_or_else(Zval::new)
            },
            "FLOAT" | "DOUBLE" | "DECIMAL" => {
                row.try_get::<Option<f64>, _>(name).unwrap_or(None).map(|v| { let mut z = Zval::new(); z.set_double(v); z }).unwrap_or_else(Zval::new)
            },
            _ => {
                row.try_get::<Option<String>, _>(name).unwrap_or(None).map(|v| { let mut z = Zval::new(); z.set_string(&v, false).unwrap(); z }).unwrap_or_else(Zval::new)
            }
        };
        map.insert(name, val).unwrap();
    }
    map.into_zval(false).unwrap_or_else(|_| Zval::new())
}

// ======================================================================================
// PostgreSQL Implementation
// ======================================================================================

#[php_class]
#[php(name = "Async\\Kernel\\DB\\PgSql")]
pub struct AsyncPgSql {
    pool: Shared<PgPool>,
}

#[php_impl]
impl AsyncPgSql {
    pub fn connect(dsn: String, max_conns: i32) -> RustFuture {
        let future = async move {
            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(max_conns as u32)
                .connect(&dsn)
                .await 
            {
                Ok(pool) => {
                    let obj = AsyncPgSql { pool: Shared::new(pool) };
                    ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                },
                Err(_) => Zval::new()
            }
        };
        RustFuture::new(future)
    }

    pub fn query(&self, sql: String, params: Option<Vec<String>>) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            let mut query = sqlx::query(&sql);
            if let Some(args) = params {
                for arg in args {
                    query = query.bind(arg);
                }
            }

            match query.fetch_all(pool.get_ref()).await {
                Ok(rows) => {
                    let mut results = ext_php_rs::types::ZendHashTable::new();
                    for row in rows {
                        results.push(pg_row_to_zval(&row)).unwrap();
                    }
                    results.into_zval(false).unwrap_or_else(|_| Zval::new())
                },
                Err(_) => Zval::new()
            }
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String, params: Option<Vec<String>>) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            let mut query = sqlx::query(&sql);
            if let Some(args) = params {
                for arg in args {
                    query = query.bind(arg);
                }
            }

            match query.execute(pool.get_ref()).await {
                Ok(done) => {
                    let mut z = Zval::new();
                    z.set_long(done.rows_affected() as i64);
                    z
                },
                Err(_) => Zval::new()
            }
        };
        RustFuture::new(future)
    }

    pub fn begin_transaction(&self) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            match pool.begin().await {
                Ok(tx) => {
                    let obj = AsyncPgSqlTransaction {
                        tx: Shared::new(Some(tx))
                    };
                    ext_php_rs::types::ZendClassObject::new(obj).into_zval(false).unwrap_or_else(|_| Zval::new())
                },
                Err(_) => Zval::new()
            }
        };
        RustFuture::new(future)
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\DB\\PgSql\\Transaction")]
pub struct AsyncPgSqlTransaction {
    tx: Shared<Option<Transaction<'static, Postgres>>>,
}

#[php_impl]
impl AsyncPgSqlTransaction {
    pub fn query(&self, sql: String, params: Option<Vec<String>>) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            if let Some(tx) = tx_rc.get_mut().as_mut() {
                let mut query = sqlx::query(&sql);
                if let Some(args) = params {
                    for arg in args {
                        query = query.bind(arg);
                    }
                }

                match query.fetch_all(&mut **tx).await {
                    Ok(rows) => {
                        let mut results = ext_php_rs::types::ZendHashTable::new();
                        for row in rows {
                            results.push(pg_row_to_zval(&row)).unwrap();
                        }
                        results.into_zval(false).unwrap_or_else(|_| Zval::new())
                    },
                    Err(_) => Zval::new()
                }
            } else {
                Zval::new()
            }
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String, params: Option<Vec<String>>) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            if let Some(tx) = tx_rc.get_mut().as_mut() {
                let mut query = sqlx::query(&sql);
                if let Some(args) = params {
                    for arg in args {
                        query = query.bind(arg);
                    }
                }

                match query.execute(&mut **tx).await {
                    Ok(done) => {
                        let mut z = Zval::new();
                        z.set_long(done.rows_affected() as i64);
                        z
                    },
                    Err(_) => Zval::new()
                }
            } else {
                Zval::new()
            }
        };
        RustFuture::new(future)
    }

    pub fn commit(&self) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            if let Some(tx) = tx_rc.get_mut().take() {
                let mut z = Zval::new();
                z.set_bool(tx.commit().await.is_ok());
                z
            } else {
                let mut z = Zval::new();
                z.set_bool(false);
                z
            }
        };
        RustFuture::new(future)
    }

    pub fn rollback(&self) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            if let Some(tx) = tx_rc.get_mut().take() {
                let mut z = Zval::new();
                z.set_bool(tx.rollback().await.is_ok());
                z
            } else {
                let mut z = Zval::new();
                z.set_bool(false);
                z
            }
        };
        RustFuture::new(future)
    }
}

fn pg_row_to_zval(row: &PgRow) -> Zval {
    let mut map = ext_php_rs::types::ZendHashTable::new();
    for col in row.columns() {
        let name = col.name();
        let type_name = col.type_info().name();
        let val: Zval = match type_name {
            "BOOL" => {
                row.try_get::<Option<bool>, _>(name).unwrap_or(None).map(|v| { let mut z = Zval::new(); z.set_bool(v); z }).unwrap_or_else(Zval::new)
            },
            "INT2" | "INT4" | "INT8" => {
                row.try_get::<Option<i64>, _>(name).unwrap_or(None).map(|v| { let mut z = Zval::new(); z.set_long(v); z }).unwrap_or_else(Zval::new)
            },
            "FLOAT4" | "FLOAT8" | "NUMERIC" => {
                row.try_get::<Option<f64>, _>(name).unwrap_or(None).map(|v| { let mut z = Zval::new(); z.set_double(v); z }).unwrap_or_else(Zval::new)
            },
            _ => {
                row.try_get::<Option<String>, _>(name).unwrap_or(None).map(|v| { let mut z = Zval::new(); z.set_string(&v, false).unwrap(); z }).unwrap_or_else(Zval::new)
            }
        };
        map.insert(name, val).unwrap();
    }
    map.into_zval(false).unwrap_or_else(|_| Zval::new())
}