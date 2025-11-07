<?php

namespace Async\IO\Adapter;

use Async\IO\ReaderAt;
use Async\IO\Reader;
use Async\IO\Seeker;

/**
 * ReaderAtAdapter adapts a Reader + Seeker to implement ReaderAt
 * Allows reading at specific offsets without affecting current position
 */
class ReaderAtAdapter implements ReaderAt
{
    private Reader $reader;
    private Seeker $seeker;

    public function __construct(Reader $reader, Seeker $seeker)
    {
        $this->reader = $reader;
        $this->seeker = $seeker;
    }

    public function readAt(int $offset, int $length): ?string
    {
        // Save current position
        $currentPos = $this->seeker->seek(0, Seeker::SEEK_CURRENT);

        // Seek to target offset
        $this->seeker->seek($offset, Seeker::SEEK_START);

        // Read data
        $data = $this->reader->read($length);

        // Restore original position
        $this->seeker->seek($currentPos, Seeker::SEEK_START);

        return $data;
    }
}
