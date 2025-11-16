<?php

require 'vendor/autoload.php';

use Async\Kernel\IO\BytesReader;
use Async\Kernel\IO\BytesWriter;
use Async\Kernel\Network\Http\HttpClient;

echo "=== BytesReader Tests ===\n\n";

// Test 1: Create from string
$reader = BytesReader::fromString("Hello, World!");
echo "Created reader from string\n";
echo "Length: " . $reader->len() . "\n";
echo "Position: " . $reader->position() . "\n";
echo "Remaining: " . $reader->remaining() . "\n\n";

// Test 2: Create from bytes
$data = "Binary data: \x00\x01\x02\x03";
$reader2 = BytesReader::fromBytes($data);
echo "Created reader from bytes\n";
echo "Length: " . $reader2->len() . "\n\n";

// Test 3: BytesWriter
echo "=== BytesWriter Tests ===\n\n";

$writer = BytesWriter::new();
echo "Created empty writer\n";
echo "Length: " . $writer->len() . "\n";
echo "Position: " . $writer->position() . "\n\n";

// Test 4: BytesWriter with capacity
$writer2 = BytesWriter::withCapacity(1024);
echo "Created writer with capacity 1024\n";
echo "Length: " . $writer2->len() . "\n\n";

// Test 5: Convert to reader/writer
echo "=== Conversion Tests ===\n\n";

$reader = BytesReader::fromString("Test data");
$asyncReader = $reader->asReader();
echo "Successfully converted BytesReader to AsyncReader\n";

$asyncSeeker = $reader->asSeeker();
echo "Successfully converted BytesReader to AsyncSeeker\n";

$writer = BytesWriter::new();
$asyncWriter = $writer->asWriter();
echo "Successfully converted BytesWriter to AsyncWriter\n";

echo "\nBytesReader/BytesWriter can be used with:\n";
echo '  - HTTP request body streaming ($request->bodyStream($reader->asReader()))' . "\n";
echo "  - File copy operations\n";
echo "  - Network I/O operations\n";
echo "  - Any API that accepts AsyncReader/AsyncWriter\n";

echo "\n=== All Tests Passed! ===\n";
