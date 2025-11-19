<?php

require 'vendor/autoload.php';

use Async\Kernel;
use Async\IO;
use Async\IO\Reader;

echo "=== Testing Dual Channel Implementation ===\n\n";

class SimpleReader implements Reader
{
    private string $data = "Test";
    private int $pos = 0;

    public function read(int $length): ?string
    {
        if ($this->pos >= strlen($this->data)) {
            echo "[Reader] EOF\n";
            return null;
        }

        $chunk = substr($this->data, $this->pos, $length);
        $this->pos += strlen($chunk);
        echo "[Reader] read({$length}) returned: '{$chunk}'\n";
        return $chunk;
    }
}

Kernel::run(function () {
    echo "1. Creating SimpleReader\n";
    $reader = new SimpleReader();

    echo "\n2. Wrapping as PhpReader\n";
    $phpReader = IO::wrapPhpReader($reader);

    echo "\n3. Converting to AsyncReader\n";
    $asyncReader = $phpReader->asReader();

    echo "\n4. Reading 2 bytes\n";
    $result = \Fiber::suspend($asyncReader->read(2));
    echo "Result: '{$result}'\n";

    echo "\n5. Reading 2 bytes again\n";
    $result2 = \Fiber::suspend($asyncReader->read(2));
    echo "Result: '{$result2}'\n";

    echo "\n✓ Test completed!\n";
});
