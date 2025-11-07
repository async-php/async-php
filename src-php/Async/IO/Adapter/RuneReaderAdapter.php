<?php

namespace Async\IO\Adapter;

use Async\IO\RuneReader;
use Async\IO\RuneScanner;
use Async\IO\Reader;

/**
 * RuneReaderAdapter adapts a Reader to implement RuneReader and RuneScanner
 * Handles UTF-8 character reading
 */
class RuneReaderAdapter implements RuneScanner
{
    private Reader $reader;
    private ?array $lastRune = null;
    private bool $hasUnread = false;

    public function __construct(Reader $reader)
    {
        $this->reader = $reader;
    }

    public function readRune(): ?array
    {
        if ($this->hasUnread && $this->lastRune !== null) {
            $this->hasUnread = false;
            return $this->lastRune;
        }

        // Read first byte to determine UTF-8 sequence length
        $first = $this->reader->read(1);
        if ($first === null || $first === '') {
            return null;
        }

        $byte = ord($first[0]);
        $size = 1;

        // Determine UTF-8 sequence length
        if ($byte < 0x80) {
            // 1-byte sequence (ASCII)
            $rune = $first;
        } elseif (($byte & 0xE0) === 0xC0) {
            // 2-byte sequence
            $size = 2;
            $rest = $this->reader->read($size - 1);
            $rune = $first . ($rest ?? '');
        } elseif (($byte & 0xF0) === 0xE0) {
            // 3-byte sequence
            $size = 3;
            $rest = $this->reader->read($size - 1);
            $rune = $first . ($rest ?? '');
        } elseif (($byte & 0xF8) === 0xF0) {
            // 4-byte sequence
            $size = 4;
            $rest = $this->reader->read($size - 1);
            $rune = $first . ($rest ?? '');
        } else {
            // Invalid UTF-8
            $rune = $first;
        }

        $this->lastRune = ['rune' => $rune, 'size' => strlen($rune)];
        return $this->lastRune;
    }

    public function unreadRune(): void
    {
        if ($this->hasUnread) {
            throw new \RuntimeException("Cannot unread rune twice");
        }
        if ($this->lastRune === null) {
            throw new \RuntimeException("No rune to unread");
        }
        $this->hasUnread = true;
    }
}
