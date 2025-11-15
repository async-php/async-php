<?php

namespace Async\Network\Http;

use Psr\Http\Message\StreamInterface;

/**
 * Simple string-based stream implementation
 *
 * Wraps a string in a PSR-7 StreamInterface for use in requests.
 */
class StringStream implements StreamInterface
{
    private string $contents;
    private int $position = 0;

    public function __construct(string $contents = '')
    {
        $this->contents = $contents;
    }

    public function __toString(): string
    {
        return $this->contents;
    }

    public function close(): void
    {
        // No-op for string streams
    }

    public function detach()
    {
        $this->contents = '';
        $this->position = 0;
        return null;
    }

    public function getSize(): ?int
    {
        return strlen($this->contents);
    }

    public function tell(): int
    {
        return $this->position;
    }

    public function eof(): bool
    {
        return $this->position >= strlen($this->contents);
    }

    public function isSeekable(): bool
    {
        return true;
    }

    public function seek($offset, $whence = SEEK_SET): void
    {
        $length = strlen($this->contents);

        switch ($whence) {
            case SEEK_SET:
                $newPosition = $offset;
                break;
            case SEEK_CUR:
                $newPosition = $this->position + $offset;
                break;
            case SEEK_END:
                $newPosition = $length + $offset;
                break;
            default:
                throw new \InvalidArgumentException('Invalid whence value');
        }

        if ($newPosition < 0 || $newPosition > $length) {
            throw new \RuntimeException('Invalid seek position');
        }

        $this->position = $newPosition;
    }

    public function rewind(): void
    {
        $this->position = 0;
    }

    public function isWritable(): bool
    {
        return false;
    }

    public function write($string): int
    {
        throw new \RuntimeException('Stream is not writable');
    }

    public function isReadable(): bool
    {
        return true;
    }

    public function read($length): string
    {
        $data = substr($this->contents, $this->position, $length);
        $this->position += strlen($data);
        return $data;
    }

    public function getContents(): string
    {
        $data = substr($this->contents, $this->position);
        $this->position = strlen($this->contents);
        return $data;
    }

    public function getMetadata($key = null)
    {
        $metadata = [
            'seekable' => true,
            'readable' => true,
            'writable' => false,
        ];

        if ($key === null) {
            return $metadata;
        }

        return $metadata[$key] ?? null;
    }
}
