<?php

namespace Async\IO;

interface ByteWriter
{
    public function writeByte(int $byte): void;
}
