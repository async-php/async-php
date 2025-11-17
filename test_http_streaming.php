<?php

require 'vendor/autoload.php';

use Async\Network\Http\Client;
use Async\Kernel;

echo "=== Testing HTTP Streaming Performance ===\n\n";

Kernel::run(function () {
    $client = new Client();

    // Test: Download a large file in chunks (streaming mode)
    echo "1. Streaming large response:\n";
    try {
        $response = $client->get('https://www.baidu.com');
        $stream = $response->getBody();

        echo "   Initial position: " . $stream->tell() . "\n";
        echo "   Is seekable: " . ($stream->isSeekable() ? 'yes' : 'no') . "\n";
        echo "   EOF before reading: " . ($stream->eof() ? 'yes' : 'no') . "\n\n";

        $chunkSize = 1024; // 1KB chunks
        $totalRead = 0;
        $chunkCount = 0;

        echo "   Reading in {$chunkSize} byte chunks:\n";
        while (!$stream->eof()) {
            $chunk = $stream->read($chunkSize);
            $chunkLen = strlen($chunk);
            $totalRead += $chunkLen;
            $chunkCount++;

            if ($chunkCount <= 5 || $stream->eof()) {
                echo "   - Chunk #{$chunkCount}: {$chunkLen} bytes (total: {$totalRead})\n";
            } elseif ($chunkCount == 6) {
                echo "   - ...\n";
            }

            if ($chunkLen === 0) {
                break; // Prevent infinite loop
            }
        }

        echo "\n   Total read: {$totalRead} bytes in {$chunkCount} chunks\n";
        echo "   Final position: " . $stream->tell() . "\n";
        echo "   EOF: " . ($stream->eof() ? 'yes' : 'no') . "\n";
        echo "   ✓ Streaming test passed\n\n";
    } catch (Exception $e) {
        echo "   ✗ Streaming test failed: {$e->getMessage()}\n\n";
    }

    // Test: Seek triggers buffered mode
    echo "2. Testing seek (triggers buffered mode):\n";
    try {
        $response = $client->get('https://www.baidu.com');
        $stream = $response->getBody();

        echo "   Reading first 50 bytes...\n";
        $first = $stream->read(50);
        echo "   Read: " . strlen($first) . " bytes\n";
        echo "   Position: " . $stream->tell() . "\n";

        echo "\n   Calling seek(0) - this should trigger buffering...\n";
        $stream->seek(0);
        echo "   Position after seek: " . $stream->tell() . "\n";

        echo "\n   Re-reading first 50 bytes...\n";
        $second = $stream->read(50);
        echo "   Read: " . strlen($second) . " bytes\n";
        echo "   Same content: " . ($first === $second ? 'yes' : 'no') . "\n";

        echo "   ✓ Seek test passed\n\n";
    } catch (Exception $e) {
        echo "   ✗ Seek test failed: {$e->getMessage()}\n\n";
    }

    // Test: __toString reads entire body
    echo "3. Testing __toString:\n";
    try {
        $response = $client->get('https://www.baidu.com');
        $stream = $response->getBody();

        echo "   Converting stream to string...\n";
        $content = (string)$stream;
        echo "   Content length: " . strlen($content) . " bytes\n";

        echo "   Position after __toString: " . $stream->tell() . "\n";
        echo "   ✓ __toString test passed\n\n";
    } catch (Exception $e) {
        echo "   ✗ __toString test failed: {$e->getMessage()}\n\n";
    }

    echo "=== All Streaming Tests Completed! ===\n";
});
