<?php

namespace Async;

/**
 * Coroutine Context Manager
 *
 * Provides per-coroutine context storage, similar to Swoole\Coroutine\Context.
 * Each coroutine has its own isolated context that can store arbitrary data.
 * The context is automatically cleaned up when the coroutine exits.
 *
 * Context data is stored in the Async\Fiber object and accessed through
 * static helper methods.
 */
class Context
{
    /**
     * Get the context for the current coroutine
     *
     * Returns the context storage for the currently executing fiber.
     * Each fiber has its own isolated context storage implemented as ArrayObject.
     *
     * @return \ArrayObject|null The context storage, or null if not in a fiber
     *
     * @example
     * ```php
     * use Async\Context;
     *
     * $fiber = new Async\Fiber(function() {
     *     $context = Context::get();
     *     $context['user_id'] = 123;
     *     $context['request_id'] = uniqid();
     *
     *     // Access in nested calls
     *     someFunction(); // can access Context::get() to get the same data
     * });
     * ```
     */
    public static function get(): ?\ArrayObject
    {
        return Fiber::getCurrentContext();
    }

    /**
     * Get the current coroutine ID
     *
     * @return int The CID of the current fiber, or 0 if not in a fiber
     */
    public static function getCid(): int
    {
        return Fiber::getCurrentCid();
    }

    /**
     * Get statistics about coroutines
     *
     * @return array{peak_count: int}
     */
    public static function stats(): array
    {
        return [
            'peak_count' => Fiber::getPeakCount(),
        ];
    }
}
