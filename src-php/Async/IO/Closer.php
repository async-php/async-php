<?php

namespace Async\IO;

interface Closer
{
    public function close(): bool;
}
