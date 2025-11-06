<?php

namespace Async\IO;

interface Seeker
{
    public const SEEK_START = 0;
    public const SEEK_CURRENT = 1;
    public const SEEK_END = 2;

    public function seek(int $offset, int $whence = self::SEEK_START): int;
}
