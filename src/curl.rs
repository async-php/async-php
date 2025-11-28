use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use ext_php_rs::convert::IntoZval;
use crate::future::RustFuture;
use crate::util::Shared;
use curl::easy::{Easy2, Handler, WriteError};

/// Send-safe option value that can be passed across thread boundaries
#[derive(Debug, Clone)]
enum CurlOptionValue {
    String(String),
    Long(i64),
    Bool(bool),
    StringArray(Vec<String>),
}

impl CurlOptionValue {
    /// Convert from Zval to CurlOptionValue
    fn from_zval(zval: &Zval) -> Option<Self> {
        if let Some(s) = zval.string() {
            Some(CurlOptionValue::String(s))
        } else if let Some(i) = zval.long() {
            Some(CurlOptionValue::Long(i))
        } else if let Some(b) = zval.bool() {
            Some(CurlOptionValue::Bool(b))
        } else if let Some(arr) = zval.array() {
            // Try to convert array to Vec<String>
            let mut strings = Vec::new();
            for (_, val) in arr.iter() {
                if let Some(s) = val.string() {
                    strings.push(s);
                }
            }
            if !strings.is_empty() {
                Some(CurlOptionValue::StringArray(strings))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Response handler that collects response body and headers
#[derive(Debug)]
struct ResponseHandler {
    body: Vec<u8>,
    headers: Vec<u8>,
}

impl ResponseHandler {
    fn new() -> Self {
        Self {
            body: Vec::new(),
            headers: Vec::new(),
        }
    }
}

impl Handler for ResponseHandler {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.body.extend_from_slice(data);
        Ok(data.len())
    }

    fn header(&mut self, data: &[u8]) -> bool {
        self.headers.extend_from_slice(data);
        true
    }
}

/// Result of a curl request
#[derive(Clone)]
struct CurlResult {
    body: Vec<u8>,
    #[allow(dead_code)]
    headers: Vec<u8>,
    response_code: u32,
    effective_url: String,
    content_type: Option<String>,
    total_time: f64,
    namelookup_time: f64,
    connect_time: f64,
    pretransfer_time: f64,
    starttransfer_time: f64,
    redirect_count: u32,
    size_upload: f64,
    size_download: f64,
    speed_download: f64,
    speed_upload: f64,
    header_size: u64,
}

impl Default for CurlResult {
    fn default() -> Self {
        Self {
            body: Vec::new(),
            headers: Vec::new(),
            response_code: 0,
            effective_url: String::new(),
            content_type: None,
            total_time: 0.0,
            namelookup_time: 0.0,
            connect_time: 0.0,
            pretransfer_time: 0.0,
            starttransfer_time: 0.0,
            redirect_count: 0,
            size_upload: 0.0,
            size_download: 0.0,
            speed_download: 0.0,
            speed_upload: 0.0,
            header_size: 0,
        }
    }
}

/// PHP-exposed curl handle for single requests
#[php_class]
#[php(name = "Async\\Kernel\\Curl\\Handle")]
#[derive(Clone)]
pub struct CurlHandle {
    /// Accumulated options until exec() is called
    options: Shared<Vec<(i64, Zval)>>,
    /// Result from last execution
    result: Shared<Option<CurlResult>>,
    /// Last error message
    error: Shared<String>,
}

#[php_impl]
impl CurlHandle {
    /// Create a new curl handle
    pub fn create() -> Self {
        Self {
            options: Shared::new(Vec::new()),
            result: Shared::new(None),
            error: Shared::new(String::new()),
        }
    }

    /// Set a curl option (accumulates until exec)
    pub fn setopt(&self, option: i64, value: &Zval) {
        self.options.get_mut().push((option, value.shallow_clone()));
    }

    /// Execute the curl request (async)
    pub fn exec(&self) -> RustFuture {
        // Extract options into Send-safe format
        let options: Vec<(i64, CurlOptionValue)> = self.options.get_ref().iter()
            .filter_map(|(opt, val)| {
                CurlOptionValue::from_zval(val).map(|v| (*opt, v))
            })
            .collect();
        let result_ref = self.result.clone();
        let error_ref = self.error.clone();

        let future = async move {
            // Execute curl in a blocking thread pool
            let result = tokio::task::spawn_blocking(move || {
                let mut easy = Easy2::new(ResponseHandler::new());

                // Apply all accumulated options
                for (opt, val) in options.iter() {
                    if let Err(e) = apply_curl_option(&mut easy, *opt, val) {
                        return Err::<Easy2<ResponseHandler>, String>(e);
                    }
                }

                // Perform the request
                easy.perform().map_err(|e| format!("Curl perform error: {}", e))?;
                Ok(easy)
            })
            .await
            .map_err(|e| {
                let err_msg = format!("Task join error: {}", e);
                err_msg
            })?;

            let response = result.map_err(|e| {
                *error_ref.get_mut() = e.clone();
                e
            })?;

            // Extract result data
            let handler = response.get_ref();
            let result = CurlResult {
                body: handler.body.clone(),
                headers: handler.headers.clone(),
                response_code: response.response_code().unwrap_or(0),
                effective_url: response.effective_url()
                    .ok()
                    .flatten()
                    .unwrap_or("")
                    .to_string(),
                content_type: response.content_type()
                    .ok()
                    .flatten()
                    .map(|s| s.to_string()),
                total_time: response.total_time()
                    .unwrap_or(std::time::Duration::ZERO)
                    .as_secs_f64(),
                namelookup_time: response.namelookup_time()
                    .unwrap_or(std::time::Duration::ZERO)
                    .as_secs_f64(),
                connect_time: response.connect_time()
                    .unwrap_or(std::time::Duration::ZERO)
                    .as_secs_f64(),
                pretransfer_time: response.pretransfer_time()
                    .unwrap_or(std::time::Duration::ZERO)
                    .as_secs_f64(),
                starttransfer_time: response.starttransfer_time()
                    .unwrap_or(std::time::Duration::ZERO)
                    .as_secs_f64(),
                redirect_count: response.redirect_count().unwrap_or(0),
                size_upload: response.upload_size().unwrap_or(0.0),
                size_download: response.download_size().unwrap_or(0.0),
                speed_download: 0.0, // Not available in this curl version
                speed_upload: 0.0,   // Not available in this curl version
                header_size: response.header_size().unwrap_or(0),
            };

            // Store result
            *result_ref.get_mut() = Some(result.clone());

            // Clear error on success
            *error_ref.get_mut() = String::new();

            // Return body as string
            String::from_utf8_lossy(&result.body).to_string().into_zval(false)
                .map_err(|e| format!("Failed to convert body to PHP string: {:?}", e))
        };

        RustFuture::new(future)
    }

    /// Get info about the last transfer
    pub fn getinfo(&self, info_id: i64) -> Zval {
        let result = self.result.get_ref();
        if let Some(res) = result.as_ref() {
            extract_info(res, info_id)
        } else {
            Zval::new()
        }
    }

    /// Reset handle to initial state
    pub fn reset(&self) {
        self.options.get_mut().clear();
        *self.result.get_mut() = None;
        *self.error.get_mut() = String::new();
    }

    /// Get last error message
    pub fn error(&self) -> String {
        self.error.get_ref().clone()
    }
}

/// PHP-exposed curl multi handle for concurrent requests
#[php_class]
#[php(name = "Async\\Kernel\\Curl\\Multi")]
#[derive(Clone)]
pub struct CurlMulti {
    handles: Shared<Vec<CurlHandle>>,
}

#[php_impl]
impl CurlMulti {
    /// Create a new multi handle
    pub fn create() -> Self {
        Self {
            handles: Shared::new(Vec::new()),
        }
    }

    /// Add a handle to the multi stack
    pub fn add_handle(&self, handle: &CurlHandle) {
        self.handles.get_mut().push(handle.clone());
    }

    /// Remove a handle from the multi stack
    pub fn remove_handle(&self, handle: &CurlHandle) -> bool {
        let handles = self.handles.get_mut();
        if let Some(pos) = handles.iter().position(|h| {
            // Compare by checking if they have the same memory address
            std::ptr::eq(h as *const _, handle as *const _)
        }) {
            handles.remove(pos);
            true
        } else {
            false
        }
    }

    /// Execute all handles concurrently
    pub fn exec_all(&self) -> RustFuture {
        let handles = self.handles.clone();

        let future = async move {
            let mut tasks = Vec::new();

            // Create Easy2 for each handle and spawn blocking tasks
            for handle in handles.get_ref().iter() {
                // Extract options into Send-safe format
                let options: Vec<(i64, CurlOptionValue)> = handle.options.get_ref().iter()
                    .filter_map(|(opt, val)| {
                        CurlOptionValue::from_zval(val).map(|v| (*opt, v))
                    })
                    .collect();

                let task = tokio::task::spawn_blocking(move || {
                    let mut easy = Easy2::new(ResponseHandler::new());

                    // Apply options from handle
                    for (opt, val) in options.iter() {
                        if let Err(e) = apply_curl_option(&mut easy, *opt, val) {
                            return Err::<Easy2<ResponseHandler>, String>(format!("Option error: {}", e));
                        }
                    }

                    // Perform the request
                    easy.perform().map_err(|e| format!("Curl perform error: {}", e))?;
                    Ok(easy)
                });

                tasks.push(task);
            }

            // Wait for all to complete concurrently
            let results = futures::future::join_all(tasks).await;

            // Process results and store in handles
            for (idx, result) in results.iter().enumerate() {
                let response_result = match result {
                    Ok(res) => res,
                    Err(e) => {
                        let err_msg = format!("Task join error on handle {}: {:?}", idx, e);
                        *handles.get_ref()[idx].error.get_mut() = err_msg.clone();
                        return Err(err_msg);
                    }
                };

                match response_result {
                    Ok(response) => {
                        let handler = response.get_ref();
                        let curl_result = CurlResult {
                            body: handler.body.clone(),
                            headers: handler.headers.clone(),
                            response_code: response.response_code().unwrap_or(0),
                            effective_url: response.effective_url()
                                .ok()
                                .flatten()
                                .unwrap_or("")
                                .to_string(),
                            content_type: response.content_type()
                                .ok()
                                .flatten()
                                .map(|s| s.to_string()),
                            total_time: response.total_time()
                                .unwrap_or(std::time::Duration::ZERO)
                                .as_secs_f64(),
                            namelookup_time: response.namelookup_time()
                                .unwrap_or(std::time::Duration::ZERO)
                                .as_secs_f64(),
                            connect_time: response.connect_time()
                                .unwrap_or(std::time::Duration::ZERO)
                                .as_secs_f64(),
                            pretransfer_time: response.pretransfer_time()
                                .unwrap_or(std::time::Duration::ZERO)
                                .as_secs_f64(),
                            starttransfer_time: response.starttransfer_time()
                                .unwrap_or(std::time::Duration::ZERO)
                                .as_secs_f64(),
                            redirect_count: response.redirect_count().unwrap_or(0),
                            size_upload: response.upload_size().unwrap_or(0.0),
                            size_download: response.download_size().unwrap_or(0.0),
                            speed_download: 0.0, // Not available in this curl version
                            speed_upload: 0.0,   // Not available in this curl version
                            header_size: response.header_size().unwrap_or(0),
                        };

                        *handles.get_ref()[idx].result.get_mut() = Some(curl_result);
                        *handles.get_ref()[idx].error.get_mut() = String::new();
                    }
                    Err(e) => {
                        let err_msg = format!("Multi exec error on handle {}: {:?}", idx, e);
                        *handles.get_ref()[idx].error.get_mut() = err_msg.clone();
                        return Err(err_msg);
                    }
                }
            }

            Ok::<i64, String>(results.len() as i64)
        };

        RustFuture::new(future)
    }
}

/// Apply a curl option to an Easy2 handle
fn apply_curl_option(easy: &mut Easy2<ResponseHandler>, option: i64, value: &CurlOptionValue) -> Result<(), String> {
    // CURLOPT constants - we'll implement the most common ones first
    match option {
        // CURLOPT_URL = 10002
        10002 => {
            if let CurlOptionValue::String(url) = value {
                easy.url(url).map_err(|e| format!("Failed to set URL: {}", e))?;
            }
        }
        // CURLOPT_FOLLOWLOCATION = 52
        52 => {
            if let CurlOptionValue::Bool(follow) = value {
                easy.follow_location(*follow).map_err(|e| format!("Failed to set follow location: {}", e))?;
            }
        }
        // CURLOPT_MAXREDIRS = 68
        68 => {
            if let CurlOptionValue::Long(max) = value {
                easy.max_redirections(*max as u32).map_err(|e| format!("Failed to set max redirects: {}", e))?;
            }
        }
        // CURLOPT_TIMEOUT = 13
        13 => {
            if let CurlOptionValue::Long(timeout) = value {
                easy.timeout(std::time::Duration::from_secs(*timeout as u64))
                    .map_err(|e| format!("Failed to set timeout: {}", e))?;
            }
        }
        // CURLOPT_CONNECTTIMEOUT = 78
        78 => {
            if let CurlOptionValue::Long(timeout) = value {
                easy.connect_timeout(std::time::Duration::from_secs(*timeout as u64))
                    .map_err(|e| format!("Failed to set connect timeout: {}", e))?;
            }
        }
        // CURLOPT_USERAGENT = 10018
        10018 => {
            if let CurlOptionValue::String(ua) = value {
                easy.useragent(ua).map_err(|e| format!("Failed to set user agent: {}", e))?;
            }
        }
        // CURLOPT_SSL_VERIFYPEER = 64
        64 => {
            if let CurlOptionValue::Bool(verify) = value {
                easy.ssl_verify_peer(*verify).map_err(|e| format!("Failed to set SSL verify peer: {}", e))?;
            }
        }
        // CURLOPT_SSL_VERIFYHOST = 81
        81 => {
            if let CurlOptionValue::Long(verify) = value {
                easy.ssl_verify_host(*verify != 0).map_err(|e| format!("Failed to set SSL verify host: {}", e))?;
            }
        }
        // CURLOPT_POST = 47
        47 => {
            if let CurlOptionValue::Bool(post) = value {
                if *post {
                    easy.post(true).map_err(|e| format!("Failed to set POST: {}", e))?;
                }
            }
        }
        // CURLOPT_CUSTOMREQUEST = 10036
        10036 => {
            if let CurlOptionValue::String(method) = value {
                easy.custom_request(method).map_err(|e| format!("Failed to set custom request: {}", e))?;
            }
        }
        // CURLOPT_POSTFIELDS = 10015
        10015 => {
            if let CurlOptionValue::String(data) = value {
                easy.post_fields_copy(data.as_bytes()).map_err(|e| format!("Failed to set post fields: {}", e))?;
            }
        }
        // CURLOPT_HTTPHEADER = 10023
        10023 => {
            if let CurlOptionValue::StringArray(headers) = value {
                let mut list = curl::easy::List::new();
                for header in headers {
                    list.append(header).map_err(|e| format!("Failed to append header: {}", e))?;
                }
                easy.http_headers(list).map_err(|e| format!("Failed to set HTTP headers: {}", e))?;
            }
        }
        // For now, silently ignore unknown options (PHP compatibility)
        _ => {
            tracing::debug!("Ignoring unsupported curl option: {}", option);
        }
    }

    Ok(())
}

/// Extract info from CurlResult based on CURLINFO constant
fn extract_info(result: &CurlResult, info_id: i64) -> Zval {
    match info_id {
        // CURLINFO_EFFECTIVE_URL = 1048577
        1048577 => result.effective_url.clone().into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_RESPONSE_CODE = 2097154
        2097154 => (result.response_code as i64).into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_TOTAL_TIME = 3145731
        3145731 => result.total_time.into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_NAMELOOKUP_TIME = 3145732
        3145732 => result.namelookup_time.into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_CONNECT_TIME = 3145733
        3145733 => result.connect_time.into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_PRETRANSFER_TIME = 3145734
        3145734 => result.pretransfer_time.into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_STARTTRANSFER_TIME = 3145735
        3145735 => result.starttransfer_time.into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_REDIRECT_COUNT = 2097172
        2097172 => (result.redirect_count as i64).into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_SIZE_UPLOAD = 3145744
        3145744 => result.size_upload.into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_SIZE_DOWNLOAD = 3145736
        3145736 => result.size_download.into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_SPEED_DOWNLOAD = 3145737
        3145737 => result.speed_download.into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_SPEED_UPLOAD = 3145738
        3145738 => result.speed_upload.into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_HEADER_SIZE = 2097163
        2097163 => (result.header_size as i64).into_zval(false).unwrap_or_else(|_| Zval::new()),
        // CURLINFO_CONTENT_TYPE = 1048594
        1048594 => result.content_type.clone()
            .unwrap_or_default()
            .into_zval(false)
            .unwrap_or_else(|_| Zval::new()),
        _ => Zval::new(),
    }
}
