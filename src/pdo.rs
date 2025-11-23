use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;

use crate::future::RustFuture;
use crate::util::Shared;

use sqlx::{Column, Row, Transaction, TypeInfo};
use sqlx::mysql::{MySqlPool, MySqlRow};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{MySql, Postgres};
use std::time::Duration;

fn secs_to_duration(secs: Option<f64>) -> Option<Duration> {
    let secs = secs?;
    if !secs.is_finite() || secs <= 0.0 {
        return None;
    }
    Some(Duration::from_secs_f64(secs))
}

fn sqlx_err_to_pdo_message(err: sqlx::Error) -> String {
    if let sqlx::Error::Database(db) = &err {
        let sqlstate = db.code().map(|c| c.to_string()).unwrap_or_else(|| "HY000".to_string());
        return format!("SQLSTATE[{}]: {}", sqlstate, db.message());
    }
    format!("SQLSTATE[HY000]: {}", err)
}

fn bind_params_mysql<'q>(
    mut query: sqlx::query::Query<'q, MySql, sqlx::mysql::MySqlArguments>,
    params: Vec<Zval>,
) -> sqlx::query::Query<'q, MySql, sqlx::mysql::MySqlArguments> {
    for p in params {
        let p = p.dereference();
        if p.is_null() {
            query = query.bind(Option::<String>::None);
        } else if let Some(v) = p.long() {
            query = query.bind(v as i64);
        } else if let Some(v) = p.double() {
            query = query.bind(v);
        } else if let Some(v) = p.bool() {
            query = query.bind(v);
        } else if let Some(v) = p.string() {
            query = query.bind(v);
        } else {
            query = query.bind(Option::<String>::None);
        }
    }
    query
}

fn bind_params_pg<'q>(
    mut query: sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>,
    params: Vec<Zval>,
) -> sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments> {
    for p in params {
        let p = p.dereference();
        if p.is_null() {
            query = query.bind(Option::<String>::None);
        } else if let Some(v) = p.long() {
            query = query.bind(v as i64);
        } else if let Some(v) = p.double() {
            query = query.bind(v);
        } else if let Some(v) = p.bool() {
            query = query.bind(v);
        } else if let Some(v) = p.string() {
            query = query.bind(v);
        } else {
            query = query.bind(Option::<String>::None);
        }
    }
    query
}

fn mysql_row_to_zval(row: &MySqlRow) -> Zval {
    let mut map = ext_php_rs::types::ZendHashTable::new();
    for col in row.columns() {
        let name = col.name();
        let type_name = col.type_info().name();
        let val: Zval = match type_name {
            "BOOLEAN" | "TINYINT" => row
                .try_get::<Option<bool>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_bool(v);
                    z
                })
                .unwrap_or_else(Zval::new),
            "SMALLINT" | "INT" | "INTEGER" | "BIGINT" => row
                .try_get::<Option<i64>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_long(v);
                    z
                })
                .unwrap_or_else(Zval::new),
            "FLOAT" | "DOUBLE" | "DECIMAL" => row
                .try_get::<Option<f64>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_double(v);
                    z
                })
                .unwrap_or_else(Zval::new),
            _ => row
                .try_get::<Option<String>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_string(&v, false).unwrap();
                    z
                })
                .unwrap_or_else(Zval::new),
        };
        map.insert(name, val).unwrap();
    }
    map.into_zval(false).unwrap_or_else(|_| Zval::new())
}

fn pg_row_to_zval(row: &PgRow) -> Zval {
    let mut map = ext_php_rs::types::ZendHashTable::new();
    for col in row.columns() {
        let name = col.name();
        let type_name = col.type_info().name();
        let val: Zval = match type_name {
            "BOOL" => row
                .try_get::<Option<bool>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_bool(v);
                    z
                })
                .unwrap_or_else(Zval::new),
            "INT2" => row
                .try_get::<Option<i16>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_long(v as i64);
                    z
                })
                .unwrap_or_else(Zval::new),
            "INT4" => row
                .try_get::<Option<i32>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_long(v as i64);
                    z
                })
                .unwrap_or_else(Zval::new),
            "INT8" => row
                .try_get::<Option<i64>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_long(v);
                    z
                })
                .unwrap_or_else(Zval::new),
            "FLOAT4" => row
                .try_get::<Option<f32>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_double(v as f64);
                    z
                })
                .unwrap_or_else(Zval::new),
            "FLOAT8" => row
                .try_get::<Option<f64>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_double(v);
                    z
                })
                .unwrap_or_else(Zval::new),
            "NUMERIC" => row
                .try_get::<Option<f64>, _>(name)
                .ok()
                .flatten()
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_double(v);
                    z
                })
                .unwrap_or_else(Zval::new),
            _ => row
                .try_get::<Option<String>, _>(name)
                .unwrap_or(None)
                .map(|v| {
                    let mut z = Zval::new();
                    z.set_string(&v, false).unwrap();
                    z
                })
                .unwrap_or_else(Zval::new),
        };
        map.insert(name, val).unwrap();
    }
    map.into_zval(false).unwrap_or_else(|_| Zval::new())
}

fn columns_meta_to_zval<C: Column>(
    cols: &[C],
) -> ext_php_rs::boxed::ZBox<ext_php_rs::types::ZendHashTable> {
    let mut out = ext_php_rs::types::ZendHashTable::new();
    for col in cols {
        let mut meta = ext_php_rs::types::ZendHashTable::new();
        meta.insert("name", col.name()).ok();
        meta.insert("native_type", col.type_info().name()).ok();
        out.push(meta).ok();
    }
    out
}

fn params_zval_to_vec(params: Option<&Zval>) -> Vec<Zval> {
    let Some(p) = params else {
        return vec![];
    };
    let Some(arr) = p.array() else {
        return vec![];
    };
    arr.iter()
        .map(|(_k, v)| v.shallow_clone())
        .collect()
}

// ======================================================================================
// MySQL (PDO kernel)
// ======================================================================================

#[php_class]
#[php(name = "Async\\Kernel\\PDO\\MySql")]
pub struct AsyncPdoMySql {
    pool: Shared<MySqlPool>,
}

#[php_impl]
impl AsyncPdoMySql {
    pub fn connect(
        dsn: String,
        max_conns: i32,
        min_conns: Option<i32>,
        connect_timeout_secs: Option<f64>,
        wait_timeout_secs: Option<f64>,
        heartbeat: Option<bool>,
        idle_time_secs: Option<f64>,
    ) -> RustFuture {
        let future = async move {
            let max_conns = max_conns.max(1) as u32;
            let mut opts = sqlx::mysql::MySqlPoolOptions::new().max_connections(max_conns);

            if let Some(min) = min_conns {
                let min = min.max(0) as u32;
                opts = opts.min_connections(min.min(max_conns));
            }
            if let Some(d) = secs_to_duration(wait_timeout_secs) {
                opts = opts.acquire_timeout(d);
            }
            if let Some(test) = heartbeat {
                opts = opts.test_before_acquire(test);
            }
            if let Some(d) = secs_to_duration(idle_time_secs) {
                opts = opts.idle_timeout(Some(d));
            }

            let pool = if let Some(d) = secs_to_duration(connect_timeout_secs) {
                tokio::time::timeout(d, opts.connect(&dsn))
                    .await
                    .map_err(|_| "SQLSTATE[HY000]: Connection pool connect timeout".to_string())?
                    .map_err(sqlx_err_to_pdo_message)?
            } else {
                opts.connect(&dsn).await.map_err(sqlx_err_to_pdo_message)?
            };

            let obj = AsyncPdoMySql { pool: Shared::new(pool) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn query(&self, sql: String, params: Option<&Zval>) -> RustFuture {
        let pool = self.pool.clone();
        let args = params_zval_to_vec(params);
        let future = async move {
            let mut query = sqlx::query(&sql);
            if !args.is_empty() {
                query = bind_params_mysql(query, args);
            }

            let rows = query
                .fetch_all(pool.get_ref())
                .await
                .map_err(sqlx_err_to_pdo_message)?;

            let cols = rows
                .first()
                .map(|first| columns_meta_to_zval(first.columns()))
                .unwrap_or_else(ext_php_rs::types::ZendHashTable::new);

            let mut result_rows = ext_php_rs::types::ZendHashTable::new();
            for row in rows {
                result_rows.push(mysql_row_to_zval(&row)).unwrap();
            }

            let mut res = ext_php_rs::types::ZendHashTable::new();
            res.insert("rows", result_rows).ok();
            res.insert("columns", cols).ok();
            res.into_zval(false).map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String, params: Option<&Zval>) -> RustFuture {
        let pool = self.pool.clone();
        let args = params_zval_to_vec(params);
        let future = async move {
            let mut query = sqlx::query(&sql);
            if !args.is_empty() {
                query = bind_params_mysql(query, args);
            }

            let done = query
                .execute(pool.get_ref())
                .await
                .map_err(sqlx_err_to_pdo_message)?;

            let mut res = ext_php_rs::types::ZendHashTable::new();
            res.insert("rows_affected", done.rows_affected() as i64).ok();
            res.insert("last_insert_id", (done.last_insert_id() as i64).to_string()).ok();
            res.into_zval(false).map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn begin_transaction(&self) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            let tx = pool
                .begin()
                .await
                .map_err(sqlx_err_to_pdo_message)?;
            let obj = AsyncPdoMySqlTransaction { tx: Shared::new(Some(tx)) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\PDO\\MySql\\Transaction")]
pub struct AsyncPdoMySqlTransaction {
    tx: Shared<Option<Transaction<'static, MySql>>>,
}

#[php_impl]
impl AsyncPdoMySqlTransaction {
    pub fn query(&self, sql: String, params: Option<&Zval>) -> RustFuture {
        let tx_rc = self.tx.clone();
        let args = params_zval_to_vec(params);
        let future = async move {
            let tx = tx_rc
                .get_mut()
                .as_mut()
                .ok_or_else(|| "SQLSTATE[25000]: Transaction already finished".to_string())?;

            let mut query = sqlx::query(&sql);
            if !args.is_empty() {
                query = bind_params_mysql(query, args);
            }

            let rows = query.fetch_all(&mut **tx).await.map_err(sqlx_err_to_pdo_message)?;
            let cols = rows
                .first()
                .map(|first| columns_meta_to_zval(first.columns()))
                .unwrap_or_else(ext_php_rs::types::ZendHashTable::new);

            let mut result_rows = ext_php_rs::types::ZendHashTable::new();
            for row in rows {
                result_rows.push(mysql_row_to_zval(&row)).unwrap();
            }

            let mut res = ext_php_rs::types::ZendHashTable::new();
            res.insert("rows", result_rows).ok();
            res.insert("columns", cols).ok();
            res.into_zval(false).map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String, params: Option<&Zval>) -> RustFuture {
        let tx_rc = self.tx.clone();
        let args = params_zval_to_vec(params);
        let future = async move {
            let tx = tx_rc
                .get_mut()
                .as_mut()
                .ok_or_else(|| "SQLSTATE[25000]: Transaction already finished".to_string())?;

            let mut query = sqlx::query(&sql);
            if !args.is_empty() {
                query = bind_params_mysql(query, args);
            }

            let done = query.execute(&mut **tx).await.map_err(sqlx_err_to_pdo_message)?;

            let mut res = ext_php_rs::types::ZendHashTable::new();
            res.insert("rows_affected", done.rows_affected() as i64).ok();
            res.insert("last_insert_id", (done.last_insert_id() as i64).to_string()).ok();
            res.into_zval(false).map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn commit(&self) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            let tx = tx_rc
                .get_mut()
                .take()
                .ok_or_else(|| "SQLSTATE[25000]: Transaction already finished".to_string())?;
            tx.commit().await.map_err(sqlx_err_to_pdo_message)?;
            Ok::<bool, String>(true)
        };
        RustFuture::new(future)
    }

    pub fn rollback(&self) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            let tx = tx_rc
                .get_mut()
                .take()
                .ok_or_else(|| "SQLSTATE[25000]: Transaction already finished".to_string())?;
            tx.rollback().await.map_err(sqlx_err_to_pdo_message)?;
            Ok::<bool, String>(true)
        };
        RustFuture::new(future)
    }
}

// ======================================================================================
// PostgreSQL (PDO kernel)
// ======================================================================================

#[php_class]
#[php(name = "Async\\Kernel\\PDO\\PgSql")]
pub struct AsyncPdoPgSql {
    pool: Shared<PgPool>,
}

#[php_impl]
impl AsyncPdoPgSql {
    pub fn connect(
        dsn: String,
        max_conns: i32,
        min_conns: Option<i32>,
        connect_timeout_secs: Option<f64>,
        wait_timeout_secs: Option<f64>,
        heartbeat: Option<bool>,
        idle_time_secs: Option<f64>,
    ) -> RustFuture {
        let future = async move {
            let max_conns = max_conns.max(1) as u32;
            let mut opts = sqlx::postgres::PgPoolOptions::new().max_connections(max_conns);

            if let Some(min) = min_conns {
                let min = min.max(0) as u32;
                opts = opts.min_connections(min.min(max_conns));
            }
            if let Some(d) = secs_to_duration(wait_timeout_secs) {
                opts = opts.acquire_timeout(d);
            }
            if let Some(test) = heartbeat {
                opts = opts.test_before_acquire(test);
            }
            if let Some(d) = secs_to_duration(idle_time_secs) {
                opts = opts.idle_timeout(Some(d));
            }

            let pool = if let Some(d) = secs_to_duration(connect_timeout_secs) {
                tokio::time::timeout(d, opts.connect(&dsn))
                    .await
                    .map_err(|_| "SQLSTATE[HY000]: Connection pool connect timeout".to_string())?
                    .map_err(sqlx_err_to_pdo_message)?
            } else {
                opts.connect(&dsn).await.map_err(sqlx_err_to_pdo_message)?
            };

            let obj = AsyncPdoPgSql { pool: Shared::new(pool) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn query(&self, sql: String, params: Option<&Zval>) -> RustFuture {
        let pool = self.pool.clone();
        let args = params_zval_to_vec(params);
        let future = async move {
            let mut query = sqlx::query(&sql);
            if !args.is_empty() {
                query = bind_params_pg(query, args);
            }

            let rows = query
                .fetch_all(pool.get_ref())
                .await
                .map_err(sqlx_err_to_pdo_message)?;

            let cols = rows
                .first()
                .map(|first| columns_meta_to_zval(first.columns()))
                .unwrap_or_else(ext_php_rs::types::ZendHashTable::new);

            let mut result_rows = ext_php_rs::types::ZendHashTable::new();
            for row in rows {
                result_rows.push(pg_row_to_zval(&row)).unwrap();
            }

            let mut res = ext_php_rs::types::ZendHashTable::new();
            res.insert("rows", result_rows).ok();
            res.insert("columns", cols).ok();
            res.into_zval(false).map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String, params: Option<&Zval>) -> RustFuture {
        let pool = self.pool.clone();
        let args = params_zval_to_vec(params);
        let future = async move {
            let mut query = sqlx::query(&sql);
            if !args.is_empty() {
                query = bind_params_pg(query, args);
            }

            let done = query
                .execute(pool.get_ref())
                .await
                .map_err(sqlx_err_to_pdo_message)?;

            let mut res = ext_php_rs::types::ZendHashTable::new();
            res.insert("rows_affected", done.rows_affected() as i64).ok();
            res.into_zval(false).map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn begin_transaction(&self) -> RustFuture {
        let pool = self.pool.clone();
        let future = async move {
            let tx = pool.begin().await.map_err(sqlx_err_to_pdo_message)?;
            let obj = AsyncPdoPgSqlTransaction { tx: Shared::new(Some(tx)) };
            ext_php_rs::types::ZendClassObject::new(obj)
                .into_zval(false)
                .map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\PDO\\PgSql\\Transaction")]
pub struct AsyncPdoPgSqlTransaction {
    tx: Shared<Option<Transaction<'static, Postgres>>>,
}

#[php_impl]
impl AsyncPdoPgSqlTransaction {
    pub fn query(&self, sql: String, params: Option<&Zval>) -> RustFuture {
        let tx_rc = self.tx.clone();
        let args = params_zval_to_vec(params);
        let future = async move {
            let tx = tx_rc
                .get_mut()
                .as_mut()
                .ok_or_else(|| "SQLSTATE[25000]: Transaction already finished".to_string())?;

            let mut query = sqlx::query(&sql);
            if !args.is_empty() {
                query = bind_params_pg(query, args);
            }

            let rows = query.fetch_all(&mut **tx).await.map_err(sqlx_err_to_pdo_message)?;
            let cols = rows
                .first()
                .map(|first| columns_meta_to_zval(first.columns()))
                .unwrap_or_else(ext_php_rs::types::ZendHashTable::new);

            let mut result_rows = ext_php_rs::types::ZendHashTable::new();
            for row in rows {
                result_rows.push(pg_row_to_zval(&row)).unwrap();
            }

            let mut res = ext_php_rs::types::ZendHashTable::new();
            res.insert("rows", result_rows).ok();
            res.insert("columns", cols).ok();
            res.into_zval(false).map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn execute(&self, sql: String, params: Option<&Zval>) -> RustFuture {
        let tx_rc = self.tx.clone();
        let args = params_zval_to_vec(params);
        let future = async move {
            let tx = tx_rc
                .get_mut()
                .as_mut()
                .ok_or_else(|| "SQLSTATE[25000]: Transaction already finished".to_string())?;

            let mut query = sqlx::query(&sql);
            if !args.is_empty() {
                query = bind_params_pg(query, args);
            }

            let done = query.execute(&mut **tx).await.map_err(sqlx_err_to_pdo_message)?;

            let mut res = ext_php_rs::types::ZendHashTable::new();
            res.insert("rows_affected", done.rows_affected() as i64).ok();
            res.into_zval(false).map_err(|e| format!("Zval conversion error: {:?}", e))
        };
        RustFuture::new(future)
    }

    pub fn commit(&self) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            let tx = tx_rc
                .get_mut()
                .take()
                .ok_or_else(|| "SQLSTATE[25000]: Transaction already finished".to_string())?;
            tx.commit().await.map_err(sqlx_err_to_pdo_message)?;
            Ok::<bool, String>(true)
        };
        RustFuture::new(future)
    }

    pub fn rollback(&self) -> RustFuture {
        let tx_rc = self.tx.clone();
        let future = async move {
            let tx = tx_rc
                .get_mut()
                .take()
                .ok_or_else(|| "SQLSTATE[25000]: Transaction already finished".to_string())?;
            tx.rollback().await.map_err(sqlx_err_to_pdo_message)?;
            Ok::<bool, String>(true)
        };
        RustFuture::new(future)
    }
}
