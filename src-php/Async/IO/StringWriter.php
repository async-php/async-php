<?php

namespace Async\IO;

interface StringWriter
{
    public function writeString(string $s): int;
}
