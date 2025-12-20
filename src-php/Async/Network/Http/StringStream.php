<?php

namespace Async\Network\Http;

use Async\Kernel\IO\BytesReader;
use Async\IO;
use Fiber;

/**
 * String-based stream implementation backed by Rust BytesReader
 *
 * This implementation uses the Rust-level BytesReader for efficient
 * in-memory operations and unified IO handling.
 *
 * Benefits over pure PHP implementation:
 * - Consistent behavior with other Rust-backed streams
 * - Efficient memory management at Rust level
 * - Seekable by default (no buffering needed)
 */
class StringStream
{
    private BytesReader $reader;
    private int $size;

    public function __construct(string $contents = '')
    {
        // Use fromBytes() since PHP strings are binary-safe
        $this->reader = BytesReader::fromBytes($contents);
        $this->size = strlen($contents);
    }

    public function __toString(): string
    {
        try {
            // Save current position
            $currentPos = $this->reader->position();

            // Reset to beginning and read all
            $this->reader->reset();
            $future = $this->reader->castTo(IO::$READ)->read($this->size);
            $contents = Fiber::suspend($future) ?? '';

            // Restore position
            $seeker = $this->reader->castTo(IO::$SEEK);
            $future = $seeker->seek($currentPos, SEEK_SET);
            Fiber::suspend($future);

            return $contents;
        } catch (\Throwable $e) {
            return '';
        }
    }

    public function close(): void
    {
        // No-op: Rust manages the memory
    }

    public function detach()
    {
        // Can't really detach from Rust-backed reader
        return null;
    }

    public function getSize(): ?int
    {
        return $this->size;
    }

    public function tell(): int
    {
        return $this->reader->position();
    }

    public function eof(): bool
    {
        return $this->reader->remaining() === 0;
    }

    public function isSeekable(): bool
    {
        return true;
    }

    public function seek($offset, $whence = SEEK_SET): void
    {
        $seeker = $this->reader->castTo(IO::$SEEK);
        $future = $seeker->seek($offset, $whence);
        Fiber::suspend($future);
    }

    public function rewind(): void
    {
        $this->reader->reset();
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
        if ($length <= 0) {
            return '';
        }

        $asyncReader = $this->reader->castTo(IO::$READ);
        $future = $asyncReader->read($length);
        $data = Fiber::suspend($future);

        return $data ?? '';
    }

    public function getContents(): string
    {
        $remaining = $this->reader->remaining();
        if ($remaining === 0) {
            return '';
        }

        return $this->read($remaining);
    }

    public function getMetadata($key = null)
    {
        $metadata = [
            'seekable' => true,
            'readable' => true,
            'writable' => false,
            'mode' => 'rust-backed',
            'type' => 'BytesReader',
        ];

        if ($key === null) {
            return $metadata;
        }

        return $metadata[$key] ?? null;
    }
}
