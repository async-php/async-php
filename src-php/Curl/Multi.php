<?php

namespace Curl;

use Async\Kernel\Curl\Multi as KernelMulti;
use Fiber;

/**
 * PHP cURL Multi Handle wrapper
 *
 * Provides concurrent execution of multiple cURL handles.
 */
class Multi
{
    private KernelMulti $kernel;
    private array $handles = [];

    public function __construct()
    {
        $this->kernel = KernelMulti::create();
    }

    /**
     * Add a handle to the multi stack
     *
     * @param Handle $handle Handle to add
     * @return int CURLM_OK on success
     */
    public function addHandle(Handle $handle): int
    {
        $id = spl_object_id($handle);
        $this->handles[$id] = $handle;
        $this->kernel->addHandle($handle->getKernel());
        return CURLM_OK;
    }

    /**
     * Remove a handle from the multi stack
     *
     * @param Handle $handle Handle to remove
     * @return int CURLM_OK on success, CURLM_BAD_HANDLE if not found
     */
    public function removeHandle(Handle $handle): int
    {
        $id = spl_object_id($handle);

        if (!isset($this->handles[$id])) {
            return CURLM_BAD_HANDLE;
        }

        $success = $this->kernel->removeHandle($handle->getKernel());
        unset($this->handles[$id]);

        return $success ? CURLM_OK : CURLM_BAD_HANDLE;
    }

    /**
     * Execute all handles concurrently (async)
     *
     * @param int|null $stillRunning Output parameter for number of handles still running
     * @return int CURLM_OK on success
     */
    public function exec(?int &$stillRunning = null): int
    {
        try {
            // Execute all handles concurrently via Fiber::suspend
            $count = Fiber::suspend($this->kernel->execAll());
            $stillRunning = 0; // All done (async operation completes all at once)
            return CURLM_OK;
        } catch (\Throwable $e) {
            return CURLM_INTERNAL_ERROR;
        }
    }

    /**
     * Get information about completed transfers
     *
     * @param int|null $msgsInQueue Output parameter for remaining messages
     * @return array|false Info array or false if no more messages
     */
    public function infoRead(?int &$msgsInQueue = null): array|false
    {
        // In our async implementation, all transfers complete at once
        // so there are no pending messages after exec()
        $msgsInQueue = 0;
        return false;
    }

    /**
     * Wait for activity on any handle (compatibility method)
     *
     * In async mode, this is handled by exec() internally.
     *
     * @param float $timeout Timeout in seconds
     * @return int Number of descriptors with activity
     */
    public function select(float $timeout = 1.0): int
    {
        // In async mode, activity is handled by the Rust actor
        return 1;
    }

    /**
     * Close the multi handle (for compatibility)
     */
    public function close(): void
    {
        $this->handles = [];
        // PHP handles cleanup automatically
    }
}
