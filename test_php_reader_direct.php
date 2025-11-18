<?php

require 'vendor/autoload.php';

use Async\Kernel;
use Async\IO;
use Async\IO\Reader;

echo "=== Testing PhpReader Direct Read ===\n\n";

/**
 * Simple PHP Reader implementation
 */
class TestReader implements Reader
{
    private string $data;
    private int $position = 0;

    public function __construct(string $data)
    {
        $this->data = $data;
        echo "[TestReader] Created with: '{$data}' (" . strlen($data) . " bytes)\n";
    }

    public function read(int $length): ?string
    {
        if ($this->position >= strlen($this->data)) {
            echo "[TestReader] read({$length}) -> EOF\n";
            return null;
        }

        $chunk = substr($this->data, $this->position, $length);
        $this->position += strlen($chunk);

        echo "[TestReader] read({$length}) -> '" . addslashes($chunk) . "' (" . strlen($chunk) . " bytes)\n";

        return $chunk;
    }
}

Kernel::run(function () {
    echo "1. Creating TestReader\n";
    $testReader = new TestReader("Hello, World from PHP!");

    echo "\n2. Wrapping as PhpReader\n";
    $phpReader = IO::wrapPhpReader($testReader);

    echo "\n3. Converting to AsyncReader\n";
    $asyncReader = $phpReader->asReader();

    echo "\n4. Reading 5 bytes\n";
    $data1 = \Fiber::suspend($asyncReader->read(5));
    echo "Got: '{$data1}'\n";

    echo "\n5. Reading 10 bytes\n";
    $data2 = \Fiber::suspend($asyncReader->read(10));
    echo "Got: '{$data2}'\n";

    echo "\n6. Reading rest\n";
    $data3 = \Fiber::suspend($asyncReader->readToEnd());
    echo "Got: '{$data3}'\n";

    echo "\n✓ Test Passed!\n";
});
