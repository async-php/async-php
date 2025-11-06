<?php

namespace Async\IO;

interface Reader
{
    public function read(int $length): ?string;
}
