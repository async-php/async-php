<?php

namespace Async\IO\Adapter;

use Async\IO\StringReader;
use Async\IO\Reader;

/**
 * StringReaderAdapter adapts a Reader to implement StringReader
 */
class StringReaderAdapter implements StringReader
{
    private Reader $reader;

    public function __construct(Reader $reader)
    {
        $this->reader = $reader;
    }

    public function readString(int $delim): string
    {
        $result = '';
        while (true) {
            $data = $this->reader->read(1);
            if ($data === null || $data === '') {
                break;
            }
            $result .= $data;
            if (ord($data[0]) === $delim) {
                break;
            }
        }
        return $result;
    }
}
