<?php

namespace Async\IO\Adapter;

use Async\IO\WriterTo;
use Async\IO\Reader;
use Async\IO\Writer;

/**
 * WriterToAdapter adapts a Reader to implement WriterTo
 * Copies data from the Reader to a Writer
 */
class WriterToAdapter implements WriterTo
{
    private Reader $reader;
    private int $bufferSize;

    public function __construct(Reader $reader, int $bufferSize = 8192)
    {
        $this->reader = $reader;
        $this->bufferSize = $bufferSize;
    }

    public function writeTo(Writer $writer): int
    {
        $totalWritten = 0;

        while (true) {
            $data = $this->reader->read($this->bufferSize);
            if ($data === null || $data === '') {
                break;
            }

            $n = $writer->write($data);
            $totalWritten += $n;

            // If we wrote less than we read, something went wrong
            if ($n < strlen($data)) {
                break;
            }
        }

        return $totalWritten;
    }
}
