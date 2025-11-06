<?php

namespace Async\IO;

interface Writer
{
    public function write(string $data): int;
    public function flush(): void;
}
