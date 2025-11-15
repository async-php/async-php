<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Kernel;
use Async\IO;

echo "=== IO Bridge Test ===\n\n";

// Test 1: PhpReader
echo "Test 1: PhpReader\n";
class TestReader {
    private $data;
    private $pos = 0;

    public function __construct($data) {
        $this->data = $data;
    }

    public function read($length) {
        if ($this->pos >= strlen($this->data)) {
            return null; // EOF
        }
        $chunk = substr($this->data, $this->pos, $length);
        $this->pos += strlen($chunk);
        echo "[PHP Reader] Read {$length} bytes, got: " . strlen($chunk) . " bytes\n";
        return $chunk;
    }
}

$fiber = new Fiber(function() {
    $reader = new TestReader("Hello, World from PHP Reader!");
    $channel = IO::spawnIO($reader);

    // Now the channel can be used by Rust PhpReader
    // For testing, let's simulate some reads
    $result1 = $channel->pop();
    echo "Request 1: " . json_encode($result1) . "\n";

    // Simulate read(10)
    $data = $reader->read(10);
    $channel->push($data);

    $result2 = $channel->pop();
    echo "Request 2: " . json_encode($result2) . "\n";

    // Simulate read(20)
    $data = $reader->read(20);
    $channel->push($data);

    // Test close command
    $result3 = $channel->pop();
    if ($result3 && $result3[0] === '__close__') {
        echo "Received close command, terminating fiber\n";
        $channel->close();
    }

    echo "Reader fiber terminated\n";
});

run($fiber);

echo "\n";

// Test 2: PhpWriter
echo "Test 2: PhpWriter\n";
class TestWriter {
    private $buffer = '';

    public function write($data) {
        $len = strlen($data);
        $this->buffer .= $data;
        echo "[PHP Writer] Wrote {$len} bytes, total: " . strlen($this->buffer) . " bytes\n";
        return $len;
    }

    public function flush() {
        echo "[PHP Writer] Flushed, buffer content: {$this->buffer}\n";
        return true;
    }

    public function close() {
        echo "[PHP Writer] Closed\n";
        return true;
    }
}

$fiber = new Fiber(function() {
    $writer = new TestWriter();
    $channel = IO::spawnIO($writer);

    // Simulate write
    $result1 = $channel->pop();
    if ($result1 && $result1[0] === 'write') {
        echo "Request: write({$result1[1][0]})\n";
        $bytes = $writer->write($result1[1][0]);
        $channel->push($bytes);
    }

    // Simulate flush
    $result2 = $channel->pop();
    if ($result2 && $result2[0] === 'flush') {
        echo "Request: flush()\n";
        $success = $writer->flush();
        $channel->push($success);
    }

    // Test close command
    $result3 = $channel->pop();
    if ($result3 && $result3[0] === '__close__') {
        echo "Received close command, terminating fiber\n";
        $channel->close();
    }

    echo "Writer fiber terminated\n";
});

run($fiber);

echo "\n";

// Test 3: PhpSeeker
echo "Test 3: PhpSeeker\n";
class TestSeeker {
    private $pos = 0;
    private $size = 1000;

    public function seek($offset, $whence) {
        switch ($whence) {
            case 0: // SEEK_SET
                $this->pos = $offset;
                break;
            case 1: // SEEK_CUR
                $this->pos += $offset;
                break;
            case 2: // SEEK_END
                $this->pos = $this->size + $offset;
                break;
        }
        echo "[PHP Seeker] Seeked to position: {$this->pos}\n";
        return $this->pos;
    }
}

$fiber = new Fiber(function() {
    $seeker = new TestSeeker();
    $channel = IO::spawnIO($seeker);

    // Simulate seek(100, SEEK_SET)
    $result1 = $channel->pop();
    if ($result1 && $result1[0] === 'seek') {
        echo "Request: seek({$result1[1][0]}, {$result1[1][1]})\n";
        $pos = $seeker->seek($result1[1][0], $result1[1][1]);
        $channel->push($pos);
    }

    // Test close command
    $result2 = $channel->pop();
    if ($result2 && $result2[0] === '__close__') {
        echo "Received close command, terminating fiber\n";
        $channel->close();
    }

    echo "Seeker fiber terminated\n";
});

run($fiber);

echo "\n";

// Test 4: PhpBufReader
echo "Test 4: PhpBufReader\n";
class TestBufReader {
    private $lines;
    private $index = 0;

    public function __construct() {
        $this->lines = [
            "First line\n",
            "Second line\n",
            "Third line\n"
        ];
    }

    public function read_line() {
        if ($this->index >= count($this->lines)) {
            return null; // EOF
        }
        $line = $this->lines[$this->index++];
        echo "[PHP BufReader] Read line: " . trim($line) . "\n";
        return $line;
    }

    public function read($length) {
        // Fallback for buffered reading
        if ($this->index >= count($this->lines)) {
            return null;
        }
        $data = substr($this->lines[$this->index], 0, $length);
        echo "[PHP BufReader] Read {$length} bytes\n";
        return $data;
    }
}

$fiber = new Fiber(function() {
    $reader = new TestBufReader();
    $channel = IO::spawnIO($reader);

    // Simulate read_line
    $result1 = $channel->pop();
    if ($result1 && $result1[0] === 'read_line') {
        echo "Request: read_line()\n";
        $line = $reader->read_line();
        $channel->push($line);
    }

    // Simulate another read_line
    $result2 = $channel->pop();
    if ($result2 && $result2[0] === 'read_line') {
        echo "Request: read_line()\n";
        $line = $reader->read_line();
        $channel->push($line);
    }

    // Test close command
    $result3 = $channel->pop();
    if ($result3 && $result3[0] === '__close__') {
        echo "Received close command, terminating fiber\n";
        $channel->close();
    }

    echo "BufReader fiber terminated\n";
});

run($fiber);

echo "\n";

// Test 5: Channel termination on closure
echo "Test 5: Channel auto-termination\n";
class TestAutoClose {
    public function read($length) {
        echo "[PHP AutoClose] Read called\n";
        return "data";
    }
}

$fiber = new Fiber(function() {
    $reader = new TestAutoClose();
    $channel = IO::spawnIO($reader);

    $result = $channel->pop();
    if ($result) {
        echo "Request: " . json_encode($result) . "\n";
        $channel->push("response");
    }

    // Close channel explicitly
    echo "Closing channel explicitly\n";
    $channel->close();

    // Next pop should return null
    $result2 = $channel->pop();
    if ($result2 === null) {
        echo "Channel closed, fiber should terminate\n";
    }

    echo "AutoClose fiber terminated\n";
});

run($fiber);

echo "\n=== All IO Bridge Tests Completed ===\n";
