<?php

namespace Async\Network\Http;

use Async\Kernel\IO\AsyncReader;
use Async\IO\Wrapper\ReaderWrapper;

/**
 * Stream implementation for HTTP response bodies
 *
 * This implementation is optimized for streaming large responses without buffering.
 * It wraps an AsyncReader for efficient streaming.
 *
 * NOTE: This stream does NOT support seeking/rewinding as it's designed for
 * forward-only streaming to handle large responses efficiently.
 */
class ReaderStream
{
    private ?ReaderWrapper $reader = null;
    private int $position = 0;
    private bool $readable = true;
    private bool $writable = false;
    private bool $eof = false;
    private ?int $contentLength = null;

    /**
     * Create a stream from an AsyncReader
     *
     * @param AsyncReader $asyncReader The async reader from response body
     * @param int|null $contentLength Optional content length from Content-Length header
     */
    public function __construct(AsyncReader $asyncReader, ?int $contentLength = null)
    {
        $this->reader = \Async\IO::wrapReader($asyncReader);
        $this->contentLength = $contentLength;
    }

    public function __toString(): string
    {
        try {
            return $this->getContents();
        } catch (\Throwable $e) {
            return '';
        }
    }

    public function close(): void
    {
        $this->reader = null;
        $this->readable = false;
        $this->eof = true;
    }

    public function detach()
    {
        $this->reader = null;
        $this->readable = false;
        $this->eof = true;
        return null;
    }

    public function getSize(): ?int
    {
        // Return content length if available from headers
        return $this->contentLength;
    }

    public function tell(): int
    {
        return $this->position;
    }

    public function eof(): bool
    {
        return $this->eof;
    }

    public function isSeekable(): bool
    {
        // This stream does not support seeking for memory efficiency
        return false;
    }

    public function seek($offset, $whence = SEEK_SET): void
    {
        throw new \RuntimeException('Stream is not seekable. This is a forward-only streaming response designed for large bodies.');
    }

    public function rewind(): void
    {
        throw new \RuntimeException('Stream is not seekable. This is a forward-only streaming response designed for large bodies.');
    }

    public function isWritable(): bool
    {
        return $this->writable;
    }

    public function write($string): int
    {
        throw new \RuntimeException('Stream is not writable');
    }

    public function isReadable(): bool
    {
        return $this->readable;
    }

    public function read($length): string
    {
        if (!$this->readable || $this->reader === null) {
            throw new \RuntimeException('Stream is not readable');
        }

        if ($length <= 0) {
            return '';
        }

        if ($this->eof) {
            return '';
        }

        try {
            $data = $this->reader->read($length);

            if ($data === null || $data === '') {
                $this->eof = true;
                return '';
            }

            $this->position += strlen($data);
            return $data;
        } catch (\Throwable $e) {
            $this->eof = true;
            return '';
        }
    }

    public function getContents(): string
    {
        if (!$this->readable || $this->reader === null) {
            throw new \RuntimeException('Stream is not readable');
        }

        if ($this->eof) {
            return '';
        }

        try {
            $result = '';
            while (true) {
                $chunk = $this->reader->read(8192);
                if ($chunk === null || $chunk === '') {
                    break;
                }
                $result .= $chunk;
                $this->position += strlen($chunk);
            }
            $this->eof = true;
            return $result;
        } catch (\Throwable $e) {
            $this->eof = true;
            return '';
        }
    }

    public function getMetadata($key = null)
    {
        $metadata = [
            'seekable' => false,
            'readable' => $this->readable,
            'writable' => $this->writable,
            'mode' => 'streaming',
        ];

        if ($key === null) {
            return $metadata;
        }

        return $metadata[$key] ?? null;
    }

    /**
     * Get the underlying AsyncReader for direct tokio usage
     *
     * This is useful when passing the stream to Rust/tokio code that can
     * consume the AsyncReader directly without intermediate buffering.
     *
     * @return \Async\Kernel\IO\AsyncReader|null
     */
    public function unwrap(): ?\Async\Kernel\IO\AsyncReader
    {
        return $this->reader?->unwrap();
    }
}
