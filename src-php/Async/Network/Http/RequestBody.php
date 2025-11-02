<?php

namespace Async\Network\Http;

use Async\Kernel\Network\Http\HttpResponseBody;
use Async\Kernel\IO\Reader;
use Fiber;

/**
 * RequestBody wraps an HTTP request body and implements the Reader interface
 */
class RequestBody implements Reader
{
    private HttpResponseBody|string|null $source;
    private int $position = 0;

    public function __construct(HttpResponseBody|string|null $source)
    {
        $this->source = $source;
    }

    /**
     * Create a RequestBody from a string
     */
    public static function fromString(string $data): self
    {
        return new self($data);
    }

    /**
     * Read up to $length bytes from the body
     *
     * @param int $length Maximum number of bytes to read
     * @return string|null Data read, or null if EOF
     */
    public function read(int $length): ?string
    {
        if ($this->source === null) {
            return null;
        }

        // If source is a string, read from it
        if (is_string($this->source)) {
            if ($this->position >= strlen($this->source)) {
                return null;
            }

            $data = substr($this->source, $this->position, $length);
            $this->position += strlen($data);

            return $data === '' ? null : $data;
        }

        // If source is HttpResponseBody, read from it asynchronously
        if ($this->source instanceof HttpResponseBody) {
            $future = $this->source->read($length);
            return Fiber::suspend($future);
        }

        return null;
    }

    /**
     * Read all remaining data from the body
     *
     * Warning: This loads the entire body into memory
     */
    public function readAll(): string
    {
        if ($this->source === null) {
            return '';
        }

        // If source is a string, return remaining data
        if (is_string($this->source)) {
            $data = substr($this->source, $this->position);
            $this->position = strlen($this->source);
            return $data;
        }

        // If source is HttpResponseBody, read all data
        if ($this->source instanceof HttpResponseBody) {
            $result = '';
            while (($chunk = $this->read(8192)) !== null) {
                $result .= $chunk;
            }
            return $result;
        }

        return '';
    }

    /**
     * Close the body stream
     */
    public function close(): bool
    {
        if ($this->source instanceof HttpResponseBody) {
            return $this->source->close();
        }

        $this->source = null;
        return true;
    }

    /**
     * Get the underlying kernel object
     */
    public function getKernel(): HttpResponseBody|string|null
    {
        return $this->source;
    }
}
