<?php

namespace Async\IO;

interface BufReader extends Reader
{
    public function readLine(): ?string;
    public function readUntil(int $delim): ?string;
}
