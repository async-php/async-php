<?php

namespace Async\IO;

interface ReaderAt
{
    public function readAt(int $offset, int $length): ?string;
}
