<?php

namespace Async\IO;

interface ByteScanner extends ByteReader
{
    public function unreadByte(): void;
    public function readBytes(int $delim): string;
}
