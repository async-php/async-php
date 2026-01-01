use ext_php_rs::types::Zval;
use sqlx::{Column, TypeInfo};
use std::time::Duration;

pub fn secs_to_duration(secs: Option<f64>) -> Option<Duration> {
    let secs = secs?;
    if !secs.is_finite() || secs <= 0.0 {
        return None;
    }
    Some(Duration::from_secs_f64(secs))
}

pub fn sqlx_err_to_pdo_message(err: sqlx::Error) -> String {
    if let sqlx::Error::Database(db) = &err {
        let sqlstate = db.code().map(|c| c.to_string()).unwrap_or_else(|| "HY000".to_string());
        return format!("SQLSTATE[{}]: {}", sqlstate, db.message());
    }
    format!("SQLSTATE[HY000]: {}", err)
}

pub fn columns_meta_to_zval<C: Column>(
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

pub fn params_zval_to_vec(params: Option<&Zval>) -> Vec<Zval> {
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
