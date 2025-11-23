<?php

namespace Curl;

use Async\Kernel\Curl\Handle as KernelHandle;
use Fiber;

/**
 * PHP cURL Handle wrapper
 *
 * This class wraps the Rust Kernel\Curl\Handle and provides the full
 * PHP cURL API compatibility. It accumulates options and delegates actual
 * HTTP operations to the Rust layer.
 */
class Handle
{
    private KernelHandle $kernel;
    private array $options = [];
    private bool $returnTransfer = false;
    private ?string $error = null;
    private int $errno = 0;

    public function __construct(?string $url = null)
    {
        $this->kernel = KernelHandle::create();
        $this->reset();

        if ($url !== null) {
            $this->setopt(CURLOPT_URL, $url);
        }
    }

    /**
     * Set a curl option
     *
     * @param int $option CURLOPT_* constant
     * @param mixed $value Option value
     * @return bool Always returns true for compatibility
     */
    public function setopt(int $option, mixed $value): bool
    {
        // Store option
        $this->options[$option] = $value;

        // Handle PHP-specific options
        if ($option === CURLOPT_RETURNTRANSFER) {
            $this->returnTransfer = (bool)$value;
            return true;
        }

        // Pass to kernel
        $this->kernel->setopt($option, $value);
        return true;
    }

    /**
     * Set multiple options at once
     *
     * @param array $options Array of CURLOPT_* => value pairs
     * @return bool True on success
     */
    public function setoptArray(array $options): bool
    {
        foreach ($options as $option => $value) {
            if (!$this->setopt($option, $value)) {
                return false;
            }
        }
        return true;
    }

    /**
     * Execute the curl request (async)
     *
     * This method suspends the current Fiber and awaits the Rust future.
     *
     * @return string|bool Response body string if CURLOPT_RETURNTRANSFER is true,
     *                     true if output was printed, false on error
     */
    public function exec(): string|bool
    {
        try {
            // Execute via Fiber::suspend to await the Rust future
            $result = Fiber::suspend($this->kernel->exec());

            // Clear error on success
            $this->error = null;
            $this->errno = 0;

            // Handle return based on CURLOPT_RETURNTRANSFER
            if ($this->returnTransfer) {
                return $result;
            } else {
                echo $result;
                return true;
            }
        } catch (\Throwable $e) {
            // Store error
            $this->error = $e->getMessage();
            $this->errno = CURLE_FAILED_INIT;
            return false;
        }
    }

    /**
     * Get information about the transfer
     *
     * @param int|null $option CURLINFO_* constant, or null for all info
     * @return mixed Info value or array of all info
     */
    public function getinfo(?int $option = null): mixed
    {
        if ($option === null) {
            // Return all info as array
            return $this->getAllInfo();
        }

        return $this->kernel->getinfo($option);
    }

    /**
     * Get the last error number
     *
     * @return int Error code (CURLE_* constant)
     */
    public function errno(): int
    {
        return $this->errno;
    }

    /**
     * Get the last error message
     *
     * @return string Error message
     */
    public function error(): string
    {
        if ($this->error !== null) {
            return $this->error;
        }

        return $this->kernel->error();
    }

    /**
     * Reset handle to initial state
     *
     * Clears all options and error state.
     */
    public function reset(): void
    {
        $this->options = self::getDefaultOptions();
        $this->returnTransfer = false;
        $this->error = null;
        $this->errno = 0;
        $this->kernel->reset();
    }

    /**
     * Close the handle (for compatibility)
     *
     * In PHP with automatic resource management, this is mostly a no-op.
     */
    public function close(): void
    {
        // PHP handles cleanup automatically
    }

    /**
     * Get all info as an associative array
     *
     * @return array All transfer info
     */
    private function getAllInfo(): array
    {
        return [
            'url' => $this->kernel->getinfo(CURLINFO_EFFECTIVE_URL),
            'content_type' => $this->kernel->getinfo(CURLINFO_CONTENT_TYPE),
            'http_code' => $this->kernel->getinfo(CURLINFO_HTTP_CODE),
            'header_size' => $this->kernel->getinfo(CURLINFO_HEADER_SIZE),
            'request_size' => $this->kernel->getinfo(CURLINFO_REQUEST_SIZE),
            'total_time' => $this->kernel->getinfo(CURLINFO_TOTAL_TIME),
            'namelookup_time' => $this->kernel->getinfo(CURLINFO_NAMELOOKUP_TIME),
            'connect_time' => $this->kernel->getinfo(CURLINFO_CONNECT_TIME),
            'pretransfer_time' => $this->kernel->getinfo(CURLINFO_PRETRANSFER_TIME),
            'starttransfer_time' => $this->kernel->getinfo(CURLINFO_STARTTRANSFER_TIME),
            'redirect_time' => $this->kernel->getinfo(CURLINFO_REDIRECT_TIME),
            'redirect_count' => $this->kernel->getinfo(CURLINFO_REDIRECT_COUNT),
            'size_upload' => $this->kernel->getinfo(CURLINFO_SIZE_UPLOAD),
            'size_download' => $this->kernel->getinfo(CURLINFO_SIZE_DOWNLOAD),
            'speed_download' => $this->kernel->getinfo(CURLINFO_SPEED_DOWNLOAD),
            'speed_upload' => $this->kernel->getinfo(CURLINFO_SPEED_UPLOAD),
        ];
    }

    /**
     * Get default curl options
     *
     * @return array Default options
     */
    private static function getDefaultOptions(): array
    {
        return [
            CURLOPT_RETURNTRANSFER => false,
            CURLOPT_FOLLOWLOCATION => true,
            CURLOPT_MAXREDIRS => 20,
            CURLOPT_TIMEOUT => 0,
            CURLOPT_CONNECTTIMEOUT => 300,
            CURLOPT_SSL_VERIFYPEER => true,
            CURLOPT_SSL_VERIFYHOST => 2,
        ];
    }

    /**
     * Get the underlying kernel handle (for Multi)
     *
     * @return KernelHandle
     * @internal
     */
    public function getKernel(): KernelHandle
    {
        return $this->kernel;
    }
}
