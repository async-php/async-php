<?php

namespace Async\IO\Adapter;

use Async\IO\ByteWriter;
use Async\IO\Writer;

/**
 * ByteWriterAdapter adapts a Writer to implement ByteWriter
 */
class ByteWriterAdapter implements ByteWriter
{
    private Writer $writer;

    public function __construct(Writer $writer)
    {
        $this->writer = $writer;
    }

    public function writeByte(int $byte): void
    {
        $this->writer->write(chr($byte & 0xFF));
    }
}
