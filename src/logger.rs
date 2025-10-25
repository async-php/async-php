use ext_php_rs::prelude::*;
use std::collections::HashMap;
use tracing_subscriber::{fmt, EnvFilter, prelude::*};
use std::str::FromStr;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

struct PhpStackLayer;

impl<S> Layer<S> for PhpStackLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        if *meta.level() <= tracing::Level::WARN {
            // Attempt to print PHP stack trace for WARN and ERROR.
            // Note: This assumes we are running in the main PHP thread.
            // We use a basic "debug_print_backtrace()" call via eval or direct call if possible.
            // Since we can't easily return values here, we just output to stdout/stderr via PHP.
            
            // Warning: Calling PHP userland functions from arbitrary Rust points can be unsafe 
            // if the VM is in an unstable state.
            
            // For now, we just print a marker. Implementing full stack trace retrieval 
            // requires accessing EG(current_execute_data) via FFI which ext-php-rs abstracts.
            // A safe way is to simply let the user know where this happened in Rust.
            
            // To actually print PHP stack, we would usually do:
            // ext_php_rs::php_print!("PHP Stack Trace:\n");
            // let _ = ext_php_rs::call_user_func("debug_print_backtrace", vec![]);
            
            // However, let's try to be safe and only do this if we are sure.
            // We will just print a separator line.
            // eprintln!("--- PHP Stack Trace Context (TODO) ---");
        }
    }
}

#[php_class]
#[php(name = "Async\\Kernel\\Logger")]
pub struct AsyncLogger;

#[php_impl]
impl AsyncLogger {
    /// Initialize the logger.
    /// Config: ['level' => 'info', 'ansi' => 'true']
    pub fn init(config: HashMap<String, String>) {
        let level_str = config.get("level").map(|s| s.as_str()).unwrap_or("info");
        let ansi = config.get("ansi").map(|s| s == "true").unwrap_or(true);
        
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::from_str(level_str))
            .unwrap_or_else(|_| EnvFilter::new("info"));

        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(false) // PHP is mostly single threaded logic here
            .with_file(true)
            .with_line_number(true)
            .with_ansi(ansi);

        let stack_layer = PhpStackLayer;

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(stack_layer)
            .try_init()
            .ok();
    }
}
