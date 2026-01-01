use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::exception::PhpException;
use ext_php_rs::convert::IntoZval; 
use ext_php_rs::builders::ModuleBuilder;

/// PHP-exposed function to compile SQL placeholders
/// Returns: ['sql' => rewritten_sql, 'placeholders' => [...]]
#[php_function]
pub fn sql_compile_placeholders(driver: String, sql: String) -> PhpResult<Zval> {
    let parsed = crate::sql_parser::parse_and_rewrite_sql(&sql, &driver)
        .map_err(|e| PhpException::default(e))?;

    // Build result array
    let mut result = ext_php_rs::types::ZendHashTable::new();

    // Add rewritten SQL
    result.insert("sql", parsed.rewritten).ok();

    // Build placeholders array
    let mut placeholders_array = ext_php_rs::types::ZendHashTable::new();
    for placeholder in parsed.placeholders {
        let mut ph_entry = ext_php_rs::types::ZendHashTable::new();
        match placeholder.kind {
            crate::sql_parser::PlaceholderKind::Positional(num) => {
                ph_entry.insert("kind", "pos").ok();
                ph_entry.insert("key", num as i64).ok();
            }
            crate::sql_parser::PlaceholderKind::Named(name) => {
                ph_entry.insert("kind", "named").ok();
                ph_entry.insert("key", name).ok();
            }
        }
        placeholders_array.push(ph_entry).ok();
    }

    result.insert("placeholders", placeholders_array).ok();

    result
        .into_zval(false)
        .map_err(|e| PhpException::default(format!("Zval conversion error: {:?}", e)))
}

/// PHP-exposed function to convert PDO DSN to SQLx URI
/// Returns: ['driver' => 'mysql'|'pgsql', 'uri' => 'mysql://...']
#[php_function]
pub fn pdo_dsn_to_sqlx(dsn: String, username: Option<String>, password: Option<String>) -> PhpResult<Zval> {
    // Parse scheme
    let colon_pos = dsn.find(':').ok_or_else(|| {
        PhpException::default(format!("Invalid DSN (missing scheme): {}", dsn))
    })?;

    let scheme = dsn[..colon_pos].to_lowercase();
    let rest = &dsn[colon_pos + 1..];

    if scheme != "mysql" && scheme != "pgsql" {
        return Err(PhpException::default(format!("Unsupported PDO driver: {}", scheme)));
    }

    // Parse DSN pairs
    let pairs = parse_dsn_pairs(rest);

    // Check for URI passthrough
    if let Some(uri) = pairs.get("__uri") {
        let mut final_uri = uri.clone();
        if scheme == "pgsql" {
            final_uri = final_uri.replace("pgsql://", "postgres://");
        }
        let mut result = ext_php_rs::types::ZendHashTable::new();
        result.insert("driver", scheme.as_str()).ok();
        result.insert("uri", final_uri).ok();
        return result.into_zval(false).map_err(|e| PhpException::default(format!("Zval conversion error: {:?}", e)));
    }

    // Extract connection parameters
    let host = pairs.get("host").cloned().unwrap_or_else(|| "localhost".to_string());
    let port = pairs.get("port").cloned();
    let dbname = pairs.get("dbname").cloned().unwrap_or_default();
    let user = username.or_else(|| pairs.get("user").cloned()).unwrap_or_default();
    let pass = password.or_else(|| pairs.get("password").or_else(|| pairs.get("pass")).cloned()).unwrap_or_default();

    // Build query parameters
    let mut query_params = std::collections::HashMap::new();
    for (k, v) in pairs.iter() {
        let lk = k.to_lowercase();
        if ["host", "port", "dbname", "user", "username", "password", "pass"].contains(&lk.as_str()) {
            continue;
        }
        let key = if scheme == "mysql" && lk == "unix_socket" {
            "socket".to_string()
        } else {
            lk
        };
        query_params.insert(key, v.clone());
    }

    // Build auth part
    let auth = if !user.is_empty() {
        let encoded_user = urlencoding::encode(&user);
        if !pass.is_empty() {
            format!("{}:{}@", encoded_user, urlencoding::encode(&pass))
        } else {
            format!("{}@", encoded_user)
        }
    } else {
        String::new()
    };

    // Build host part
    let mut host_part = host.clone();
    if scheme == "pgsql" && host.starts_with('/') {
        // Unix socket mode for PostgreSQL
        query_params.insert("host".to_string(), host);
        host_part = "localhost".to_string();
    }
    if let Some(p) = port {
        if !p.is_empty() {
            host_part = format!("{}:{}", host_part, p);
        }
    }

    // Build path and query string
    let path = if !dbname.is_empty() {
        format!("/{}", urlencoding::encode(&dbname))
    } else {
        String::new()
    };

    let qs = if !query_params.is_empty() {
        let pairs: Vec<String> = query_params.iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect();
        format!("?{}", pairs.join("&"))
    } else {
        String::new()
    };

    // Build final URI
    let uri_scheme = if scheme == "pgsql" { "postgres" } else { "mysql" };
    let uri = format!("{}://{}{}{}{}", uri_scheme, auth, host_part, path, qs);

    let mut result = ext_php_rs::types::ZendHashTable::new();
    result.insert("driver", scheme.as_str()).ok();
    result.insert("uri", uri).ok();
    result.into_zval(false).map_err(|e| PhpException::default(format!("Zval conversion error: {:?}", e)))
}

fn parse_dsn_pairs(rest: &str) -> std::collections::HashMap<String, String> {
    let rest = rest.trim();
    if rest.is_empty() {
        return std::collections::HashMap::new();
    }

    // Check for URI passthrough
    if rest.contains("://") {
        let mut map = std::collections::HashMap::new();
        map.insert("__uri".to_string(), rest.to_string());
        return map;
    }

    let mut result = std::collections::HashMap::new();
    for chunk in rest.split(';') {
        let chunk = chunk.trim();
        if chunk.is_empty() {
            continue;
        }
        if let Some(eq_pos) = chunk.find('=') {
            let k = chunk[..eq_pos].trim();
            let v = chunk[eq_pos + 1..].trim();
            result.insert(k.to_string(), v.to_string());
        } else {
            result.insert(chunk.to_string(), String::new());
        }
    }
    result
}

/// PHP-exposed function to normalize parameter keys
/// Removes leading ':' from string parameters
#[php_function]
pub fn pdo_normalize_param_key(param: &Zval) -> Zval {
    if let Some(s) = param.string() {
        let normalized = s.trim_start_matches(':');
        let mut result = Zval::new();
        result.set_string(normalized, false).ok();
        result
    } else if let Some(i) = param.long() {
        let mut result = Zval::new();
        result.set_long(i);
        result
    } else {
        param.shallow_clone()
    }
}

pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    module
        .function(wrap_function!(sql_compile_placeholders))
        .function(wrap_function!(pdo_dsn_to_sqlx))
        .function(wrap_function!(pdo_normalize_param_key))
}