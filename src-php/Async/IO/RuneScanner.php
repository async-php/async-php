<?php

namespace Async\IO;

interface RuneScanner extends RuneReader
{
    public function unreadRune(): void;
}
