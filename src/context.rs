use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;

type ContextMap = HashMap<String, Zval>;

tokio::task_local! {
    static TASK_CONTEXT: RefCell<ContextMap>;
}

thread_local! {
    static THREAD_CONTEXT: RefCell<ContextMap> = RefCell::new(HashMap::new());
}

fn get_from_cell(cell: &RefCell<ContextMap>, id: &str, default: Option<&Zval>) -> Zval {
    let map = cell.borrow();
    if let Some(val) = map.get(id) {
        val.shallow_clone()
    } else {
        default
            .map(|v| v.shallow_clone())
            .unwrap_or_else(Zval::new)
    }
}

fn set_in_cell(cell: &RefCell<ContextMap>, id: &str, value: &Zval) {
    cell.borrow_mut().insert(id.to_string(), value.shallow_clone());
}

pub fn scope<F>(future: F) -> impl Future<Output = F::Output>
where
    F: Future,
{
    TASK_CONTEXT.scope(RefCell::new(HashMap::new()), future)
}

pub fn spawn_local<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    tokio::task::spawn_local(scope(future))
}

#[php_class]
#[php(name = "Async\\Kernel\\Context")]
pub struct AsyncContext;

#[php_impl]
impl AsyncContext {
    #[php(optional = "default")]
    pub fn get(id: String, default: Option<&Zval>) -> Zval {
        match TASK_CONTEXT.try_with(|cell| get_from_cell(cell, &id, default)) {
            Ok(val) => val,
            Err(_) => THREAD_CONTEXT.with(|cell| get_from_cell(cell, &id, default)),
        }
    }

    pub fn set(id: String, value: &Zval) {
        if TASK_CONTEXT.try_with(|cell| set_in_cell(cell, &id, value)).is_err() {
            THREAD_CONTEXT.with(|cell| set_in_cell(cell, &id, value));
        }
    }
}
