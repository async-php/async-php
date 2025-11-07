<?php

namespace Async\IO\Adapter;

use Async\IO\ByteReader;
use Async\IO\ByteScanner;
use Async\IO\Reader;

/**
 * ByteReaderAdapter adapts a Reader to implement ByteReader and ByteScanner
 */
class ByteReaderAdapter implements ByteScanner
{
    private Reader $reader;
    private ?int $lastByte = null;
    private bool $hasUnread = false;

    public function __construct(Reader $reader)
    {
        $this->reader = $reader;
    }

    public function readByte(): int
    {
        if ($this->hasUnread && $this->lastByte !== null) {
            $this->hasUnread = false;
            return $this->lastByte;
        }

        $data = $this->reader->read(1);
        if ($data === null || $data === '') {
            return -1;
        }

        $this->lastByte = ord($data[0]);
        return $this->lastByte;
    }

    public function unreadByte(): void
    {
        if ($this->hasUnread) {
            throw new \RuntimeException("Cannot unread byte twice");
        }
        if ($this->lastByte === null) {
            throw new \RuntimeException("No byte to unread");
        }
        $this->hasUnread = true;
    }

    public function readBytes(int $delim): string
    {
        $result = '';
        while (true) {
            $byte = $this->readByte();
            if ($byte === -1) {
                break;
            }
            $result .= chr($byte);
            if ($byte === $delim) {
                break;
            }
        }
        return $result;
    }
}
