use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use std::cell::RefCell;
use std::cell::Cell;
use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::task::LocalSet;

type ContextMap = HashMap<String, Zval>;

tokio::task_local! {
    static TASK_CONTEXT: RefCell<ContextMap>;
    static TASK_FIBER_ID: u64;
}

thread_local! {
    static THREAD_CONTEXT: RefCell<ContextMap> = RefCell::new(HashMap::new());
    static LOCAL_SET_PTR: Cell<*const LocalSet> = Cell::new(std::ptr::null());
}

static NEXT_FIBER_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_fiber_id() -> u64 {
    NEXT_FIBER_ID.fetch_add(1, Ordering::Relaxed)
}

pub(crate) struct LocalSetGuard;

impl Drop for LocalSetGuard {
    fn drop(&mut self) {
        LOCAL_SET_PTR.with(|cell| cell.set(std::ptr::null()));
    }
}

pub(crate) fn set_current_local_set(local: &LocalSet) -> LocalSetGuard {
    LOCAL_SET_PTR.with(|cell| cell.set(local as *const LocalSet));
    LocalSetGuard
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
    scope_with_fiber_id(0, future)
}

pub fn scope_with_fiber_id<F>(fiber_id: u64, future: F) -> impl Future<Output = F::Output>
where
    F: Future,
{
    TASK_FIBER_ID.scope(fiber_id, TASK_CONTEXT.scope(RefCell::new(HashMap::new()), future))
}

pub fn spawn_local<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    LOCAL_SET_PTR.with(|cell| {
        let ptr = cell.get();
        if ptr.is_null() {
            tokio::task::spawn_local(scope(future))
        } else {
            // SAFETY: `ptr` is set/cleared by `set_current_local_set` and only used on the same thread.
            unsafe { (&*ptr).spawn_local(scope(future)) }
        }
    })
}

pub fn spawn_local_with_fiber_id<F>(fiber_id: u64, future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
{
    LOCAL_SET_PTR.with(|cell| {
        let ptr = cell.get();
        if ptr.is_null() {
            tokio::task::spawn_local(scope_with_fiber_id(fiber_id, future))
        } else {
            // SAFETY: `ptr` is set/cleared by `set_current_local_set` and only used on the same thread.
            unsafe { (&*ptr).spawn_local(scope_with_fiber_id(fiber_id, future)) }
        }
    })
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
