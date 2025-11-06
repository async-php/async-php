<?php

namespace Async\IO;

interface WriterAt
{
    public function writeAt(int $offset, string $data): int;
}
