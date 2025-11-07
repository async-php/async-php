<?php

namespace Async\IO\Adapter;

use Async\IO\StringWriter;
use Async\IO\Writer;

/**
 * StringWriterAdapter adapts a Writer to implement StringWriter
 */
class StringWriterAdapter implements StringWriter
{
    private Writer $writer;

    public function __construct(Writer $writer)
    {
        $this->writer = $writer;
    }

    public function writeString(string $s): int
    {
        return $this->writer->write($s);
    }
}
