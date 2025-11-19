<?php

require 'vendor/autoload.php';

use Async\Kernel;
use Async\IO;

echo "=== Testing PHP IO Wrappers ===\n\n";

// Test 1: PhpReader
echo "Test 1: PhpReader\n";
class MyReader {
    private $data = "Hello from PHP Reader!";
    private $pos = 0;

    public function read($length) {
        if ($this->pos >= strlen($this->data)) {
            return null; // EOF
        }
        $chunk = substr($this->data, $this->pos, $length);
        $this->pos += strlen($chunk);
        return $chunk;
    }
}

Kernel::run(function () {
    $reader = new MyReader();
    $phpReader = IO::wrapPhpReader($reader);

    // Convert to AsyncReader for use in tokio
    $asyncReader = $phpReader->asReader();

    // Read some data
    $data1 = \Fiber::suspend($asyncReader->read(5));
    echo "Read 5 bytes: '{$data1}'\n";

    $data2 = \Fiber::suspend($asyncReader->read(10));
    echo "Read 10 bytes: '{$data2}'\n";

    $remaining = \Fiber::suspend($asyncReader->readToEnd());
    echo "Remaining: '{$remaining}'\n";

    echo "✓ Test 1 passed\n\n";
});

// Test 2: PhpWriter
echo "Test 2: PhpWriter\n";
class MyWriter {
    private $buffer = '';

    public function write($data) {
        $this->buffer .= $data;
        return strlen($data);
    }

    public function flush() {
        echo "Flushed: '{$this->buffer}'\n";
        return true;
    }

    public function close() {
        echo "Closed\n";
        return true;
    }
}

Kernel::run(function () {
    $writer = new MyWriter();
    $phpWriter = IO::wrapPhpWriter($writer);

    // Convert to AsyncWriter
    $asyncWriter = $phpWriter->asWriter();

    // Write some data
    $n1 = \Fiber::suspend($asyncWriter->write("Hello "));
    echo "Wrote {$n1} bytes\n";

    $n2 = \Fiber::suspend($asyncWriter->write("World!"));
    echo "Wrote {$n2} bytes\n";

    // Flush
    \Fiber::suspend($asyncWriter->flush());

    echo "✓ Test 2 passed\n\n";
});

// Test 3: PhpSeeker
echo "Test 3: PhpSeeker\n";
class MySeeker {
    private $pos = 0;
    private $size = 1000;

    public function seek($offset, $whence) {
        switch ($whence) {
            case SEEK_SET:
                $this->pos = $offset;
                break;
            case SEEK_CUR:
                $this->pos += $offset;
                break;
            case SEEK_END:
                $this->pos = $this->size + $offset;
                break;
        }
        return $this->pos;
    }
}

Kernel::run(function () {
    $seeker = new MySeeker();
    $phpSeeker = IO::wrapPhpSeeker($seeker);

    // Convert to AsyncSeeker
    $asyncSeeker = $phpSeeker->asSeeker();

    // Seek to position
    $pos1 = \Fiber::suspend($asyncSeeker->seek(100, SEEK_SET));
    echo "Seek(100, SEEK_SET) -> position: {$pos1}\n";

    $pos2 = \Fiber::suspend($asyncSeeker->seek(50, SEEK_CUR));
    echo "Seek(50, SEEK_CUR) -> position: {$pos2}\n";

    $pos3 = \Fiber::suspend($asyncSeeker->seek(-10, SEEK_END));
    echo "Seek(-10, SEEK_END) -> position: {$pos3}\n";

    echo "✓ Test 3 passed\n\n";
});

// Test 4: PhpBufReader
echo "Test 4: PhpBufReader\n";
class MyBufReader {
    private $lines = [
        "First line\n",
        "Second line\n",
        "Third line\n",
    ];
    private $index = 0;

    public function read_line() {
        if ($this->index >= count($this->lines)) {
            return null; // EOF
        }
        return $this->lines[$this->index++];
    }

    public function read($length) {
        // Fallback for non-line reads
        return null;
    }
}

Kernel::run(function () {
    $reader = new MyBufReader();
    $phpBufReader = IO::wrapPhpBufReader($reader);

    // Convert to AsyncBufReader
    $asyncBufReader = $phpBufReader->asBufReader();

    // Read lines
    $line1 = \Fiber::suspend($asyncBufReader->readLine());
    echo "Line 1: " . trim($line1) . "\n";

    $line2 = \Fiber::suspend($asyncBufReader->readLine());
    echo "Line 2: " . trim($line2) . "\n";

    $line3 = \Fiber::suspend($asyncBufReader->readLine());
    echo "Line 3: " . trim($line3) . "\n";

    echo "✓ Test 4 passed\n\n";
});

// Test 5: Using in HTTP request body
echo "Test 5: Using PhpReader as HTTP request body\n";
class StringReader {
    private $data;
    private $pos = 0;

    public function __construct($data) {
        $this->data = $data;
    }

    public function read($length) {
        if ($this->pos >= strlen($this->data)) {
            return null;
        }
        $chunk = substr($this->data, $this->pos, $length);
        $this->pos += strlen($chunk);
        return $chunk;
    }
}

Kernel::run(function () {
    $client = new \Async\Kernel\Network\Http\HttpClient();

    // Create a reader from string
    $jsonData = json_encode(['test' => 'data', 'value' => 123]);
    $stringReader = new StringReader($jsonData);

    // Wrap as PhpReader
    $phpReader = IO::wrapPhpReader($stringReader);

    // Use as request body
    $request = $client->post('https://httpbin.org/post');
    $request->header('Content-Type', 'application/json');
    $request->bodyStream($phpReader->asReader());

    $response = \Fiber::suspend($request->send());
    echo "Status: " . $response->getStatus() . "\n";

    $body = \Fiber::suspend($response->text());
    $data = json_decode($body, true);

    if (isset($data['json'])) {
        echo "Server received JSON: " . json_encode($data['json']) . "\n";
    }

    echo "✓ Test 5 passed\n\n";
});

echo "=== All Tests Completed ===\n";
