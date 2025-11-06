<?php

namespace Async\IO;

interface StringReader
{
    public function readString(int $delim): string;
}
