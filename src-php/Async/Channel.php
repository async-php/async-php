<?php

namespace Async;

use Async\Kernel\Channel as KernelChannel;
use Fiber;
use RustFuture;

class Channel
{
    private KernelChannel $inner;

    /**
     * Create a new channel
     *
     * @param int|null $capacity Buffer capacity (null for unlimited, positive integer for bounded)
     */
    public function __construct(?int $capacity = null)
    {
        $this->inner = new KernelChannel($capacity);
    }

    /**
     * Push a value to the channel
     *
     * @param mixed $data The value to push
     * @param float|null $timeout Timeout in seconds (null for blocking wait)
     * @return bool True on success, false on failure
     */
    public function push(mixed $data, ?float $timeout = null): bool
    {
        $future = $this->inner->send($data, $timeout);
        return Fiber::suspend($future);
    }

    /**
     * Pop a value from the channel
     *
     * @param float|null $timeout Timeout in seconds (null for blocking wait)
     * @return mixed The received value, or null on failure
     */
    public function pop(?float $timeout = null): mixed
    {
        $future = $this->inner->recv($timeout);
        return Fiber::suspend($future);
    }

    /**
     * Check if the channel is empty
     *
     * @return bool True if empty
     */
    public function isEmpty(): bool
    {
        return $this->inner->isEmpty();
    }

    /**
     * Check if the channel is full
     *
     * @return bool True if full
     */
    public function isFull(): bool
    {
        return $this->inner->isFull();
    }

    /**
     * Get the current number of items in the channel
     *
     * @return int Current length
     */
    public function length(): int
    {
        return $this->inner->length();
    }

    /**
     * Close the channel
     *
     * @return bool True on success
     */
    public function close(): bool
    {
        return $this->inner->close();
    }

    /**
     * Check if the channel is closed
     *
     * @return bool True if closed
     */
    public function isClosed(): bool
    {
        return $this->inner->isClosed();
    }

    /**
     * Get channel statistics
     *
     * @return array{capacity: int, length: int, is_empty: bool, is_full: bool, is_closed: bool}
     */
    public function stat(): array
    {
        return $this->inner->stat();
    }

    /**
     * Get the underlying KernelChannel
     */
    public function unwrap(): KernelChannel
    {
        return $this->inner;
    }
}
