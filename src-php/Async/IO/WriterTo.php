<?php

namespace Async\IO;

interface WriterTo
{
    public function writeTo(Writer $writer): int;
}
