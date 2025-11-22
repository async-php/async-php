<?php

namespace Async;

/**
 * Coroutine Context Manager
 *
 * Provides per-coroutine context storage, similar to Swoole\Coroutine\Context.
 * Each coroutine has its own isolated context that can store arbitrary data.
 * The context is automatically cleaned up when the coroutine exits.
 *
 * Backed by the extension (Async\Kernel\Context) using per-task storage,
 * so each spawned fiber/task gets an isolated context.
 */
class Context
{
    public static function get(string $id, mixed $default = null): mixed
    {
        return \Async\Kernel\Context::get($id, $default);
    }

    public static function set(string $id, mixed $value): void
    {
        \Async\Kernel\Context::set($id, $value);
    }
}
