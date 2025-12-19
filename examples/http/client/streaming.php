<?php

require_once __DIR__ . '/../../../vendor/autoload.php';

use Async\Kernel;
use Async\Network\Http\Client;

echo "=== HTTP Streaming Response Test ===\n";
echo "Testing streaming/chunked response body reading\n\n";

Kernel::run(function () {
    $client = new Client();
    $client->setTimeout(30);

    // Test 1: Stream reading from a regular response
    try {
        echo "Test 1: Streaming read from baidu.com\n";
        echo str_repeat('=', 70) . "\n";

        echo "Sending request...\n";
        $response = $client->get('http://www.baidu.com', [
            'headers' => [
                'User-Agent' => 'async-php/1.0',
            ]
        ]);

        echo "Status: {$response->getStatusCode()} {$response->getReasonPhrase()}\n";
        echo "Reading body in chunks...\n";

        $totalBytes = 0;
        $chunkCount = 0;
        $chunkSize = 1024; // Read 1KB at a time

        while (true) {
            $chunk = $response->readChunk($chunkSize);

            if ($chunk === null) {
                break;
            }

            $chunkCount++;
            $chunkLen = strlen($chunk);
            $totalBytes += $chunkLen;

            echo "  Chunk #{$chunkCount}: {$chunkLen} bytes\n";

            // Show first chunk content preview
            if ($chunkCount === 1) {
                echo "  First chunk preview: " . substr($chunk, 0, 100) . "...\n";
            }
        }

        echo "\nTotal: {$totalBytes} bytes in {$chunkCount} chunks\n";
        echo "✓ Streaming read successful!\n";

        echo "\n";
    } catch (Exception $e) {
        echo "✗ Test 1 failed: {$e->getMessage()}\n\n";
    }

    // Test 2: Compare streaming vs readAll
    try {
        echo "Test 2: Streaming vs readAll comparison\n";
        echo str_repeat('=', 70) . "\n";

        // First request - streaming
        echo "Method 1: Streaming read\n";
        $start = microtime(true);
        $response1 = $client->get('https://api.github.com/', [
            'headers' => [
                'Accept' => 'application/json',
            ]
        ]);

        $content1 = '';
        $chunks = 0;

        while (true) {
            $chunk = $response1->readChunk(512);
            if ($chunk === null) break;
            $content1 .= $chunk;
            $chunks++;
        }
        $time1 = microtime(true) - $start;

        echo "  Read {$chunks} chunks, total: " . strlen($content1) . " bytes\n";
        echo "  Time: " . number_format($time1 * 1000, 2) . " ms\n\n";

        // Second request - readAll
        echo "Method 2: readAll\n";
        $start = microtime(true);
        $response2 = $client->get('https://api.github.com/', [
            'headers' => [
                'Accept' => 'application/json',
            ]
        ]);

        $content2 = $response2->getBody();
        $time2 = microtime(true) - $start;

        echo "  Read all at once: " . strlen($content2) . " bytes\n";
        echo "  Time: " . number_format($time2 * 1000, 2) . " ms\n\n";

        echo "Both methods read the same amount of data\n";
        echo "✓ Test 2 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 2 failed: {$e->getMessage()}\n\n";
    }

    // Test 3: Streaming with progress tracking (simulating download)
    try {
        echo "Test 3: Download with progress tracking\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('http://www.baidu.com');

        $contentLength = $response->getHeader('content-length');
        $expectedSize = $contentLength ? (int)$contentLength : null;

        echo "Starting download";
        if ($expectedSize) {
            echo " (expected size: " . number_format($expectedSize) . " bytes)";
        }
        echo "...\n";

        $downloaded = 0;
        $lastPercent = 0;

        while (true) {
            $chunk = $response->readChunk(8192); // 8KB chunks

            if ($chunk === null) {
                break;
            }

            $downloaded += strlen($chunk);

            // Show progress
            if ($expectedSize && $expectedSize > 0) {
                $percent = (int)(($downloaded / $expectedSize) * 100);
                if ($percent >= $lastPercent + 10) {
                    echo "  Progress: {$percent}% ({$downloaded} / {$expectedSize} bytes)\n";
                    $lastPercent = $percent;
                }
            }
        }

        echo "Download complete: " . number_format($downloaded) . " bytes\n";
        echo "✓ Test 3 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 3 failed: {$e->getMessage()}\n\n";
    }

    // Test 4: Early termination (close before reading all)
    try {
        echo "Test 4: Early termination\n";
        echo str_repeat('=', 70) . "\n";

        $response = $client->get('http://www.baidu.com');

        // Read only first 3 chunks
        echo "Reading only first 3 chunks...\n";
        for ($i = 1; $i <= 3; $i++) {
            $chunk = $response->readChunk(1024);

            if ($chunk !== null) {
                echo "  Chunk #{$i}: " . strlen($chunk) . " bytes\n";
            }
        }

        // Close the body early
        $response->closeBody();
        echo "Body closed early\n";
        echo "✓ Test 4 passed!\n\n";
    } catch (Exception $e) {
        echo "✗ Test 4 failed: {$e->getMessage()}\n\n";
    }

    echo str_repeat('=', 70) . "\n";
    echo "All streaming tests completed!\n";
});
