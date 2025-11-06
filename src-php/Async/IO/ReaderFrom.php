<?php

namespace Async\IO;

interface ReaderFrom
{
    public function readFrom(Reader $reader): int;
}
