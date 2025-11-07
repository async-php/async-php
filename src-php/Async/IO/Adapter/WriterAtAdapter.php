<?php

namespace Async\IO\Adapter;

use Async\IO\WriterAt;
use Async\IO\Writer;
use Async\IO\Seeker;

/**
 * WriterAtAdapter adapts a Writer + Seeker to implement WriterAt
 * Allows writing at specific offsets without affecting current position
 */
class WriterAtAdapter implements WriterAt
{
    private Writer $writer;
    private Seeker $seeker;

    public function __construct(Writer $writer, Seeker $seeker)
    {
        $this->writer = $writer;
        $this->seeker = $seeker;
    }

    public function writeAt(int $offset, string $data): int
    {
        // Save current position
        $currentPos = $this->seeker->seek(0, Seeker::SEEK_CURRENT);

        // Seek to target offset
        $this->seeker->seek($offset, Seeker::SEEK_START);

        // Write data
        $n = $this->writer->write($data);

        // Restore original position
        $this->seeker->seek($currentPos, Seeker::SEEK_START);

        return $n;
    }
}
