<?php

namespace Async\Network\Http;

use Async\Kernel\IO\ReadCloser;

/**
 * StringReader implements ReadCloser for string data
 * This allows sending string bodies in HTTP requests
 */
class StringReader implements ReadCloser
{
    private string $data;
    private int $position = 0;

    public function __construct(string $data = '')
    {
        $this->data = $data;
    }

    /**
     * Read up to $length bytes from the string
     *
     * @param int $length Maximum number of bytes to read
     * @return string|null Data read, or null if EOF
     */
    public function read(int $length): ?string
    {
        if ($this->position >= strlen($this->data)) {
            return null;
        }

        $chunk = substr($this->data, $this->position, $length);
        $this->position += strlen($chunk);

        return $chunk === '' ? null : $chunk;
    }

    /**
     * Close the reader
     */
    public function close(): bool
    {
        $this->data = '';
        $this->position = 0;
        return true;
    }

    /**
     * Create a StringReader from string data
     */
    public static function fromString(string $data): self
    {
        return new self($data);
    }
}
