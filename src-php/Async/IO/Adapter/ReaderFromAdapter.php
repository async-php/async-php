<?php

namespace Async\IO\Adapter;

use Async\IO\ReaderFrom;
use Async\IO\Reader;
use Async\IO\Writer;

/**
 * ReaderFromAdapter adapts a Writer to implement ReaderFrom
 * Copies data from a Reader into the Writer
 */
class ReaderFromAdapter implements ReaderFrom
{
    private Writer $writer;
    private int $bufferSize;

    public function __construct(Writer $writer, int $bufferSize = 8192)
    {
        $this->writer = $writer;
        $this->bufferSize = $bufferSize;
    }

    public function readFrom(Reader $reader): int
    {
        $totalWritten = 0;

        while (true) {
            $data = $reader->read($this->bufferSize);
            if ($data === null || $data === '') {
                break;
            }

            $n = $this->writer->write($data);
            $totalWritten += $n;

            // If we wrote less than we read, something went wrong
            if ($n < strlen($data)) {
                break;
            }
        }

        return $totalWritten;
    }
}
