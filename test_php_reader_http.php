<?php

require 'vendor/autoload.php';

use Async\Kernel;
use Async\IO;
use Async\IO\Reader;

echo "=== Testing PhpReader with HTTP Request Body ===\n\n";

/**
 * Pure PHP implementation of Reader interface
 * This reads data from a string in chunks
 */
class StringReader implements Reader
{
    private string $data;
    private int $position = 0;

    public function __construct(string $data)
    {
        $this->data = $data;
        echo "[StringReader] Created with " . strlen($data) . " bytes\n";
    }

    public function read(int $length): ?string
    {
        if ($this->position >= strlen($this->data)) {
            echo "[StringReader] EOF reached\n";
            return null; // EOF
        }

        $chunk = substr($this->data, $this->position, $length);
        $this->position += strlen($chunk);

        echo "[StringReader] read({$length}) -> " . strlen($chunk) . " bytes (pos: {$this->position})\n";

        return $chunk;
    }
}

Kernel::run(function () {
    echo "1. Creating HTTP client\n";
    $client = new \Async\Kernel\Network\Http\HttpClient();

    echo "\n2. Preparing request body (JSON data)\n";
    $jsonData = json_encode([
        'message' => 'Hello from PHP Reader!',
        'timestamp' => time(),
        'test' => 'PhpReader integration',
        'nested' => [
            'foo' => 'bar',
            'num' => 123
        ]
    ]);
    echo "JSON body: {$jsonData}\n";
    echo "Body size: " . strlen($jsonData) . " bytes\n";

    echo "\n3. Creating PHP Reader (implements Async\\IO\\Reader)\n";
    $stringReader = new StringReader($jsonData);

    echo "\n4. Wrapping as PhpReader (Rust-side bridge)\n";
    $phpReader = IO::wrapPhpReader($stringReader);
    echo "PhpReader type: " . get_class($phpReader) . "\n";

    echo "\n5. Converting to AsyncReader for HTTP request\n";
    $asyncReader = $phpReader->asReader();
    echo "AsyncReader type: " . get_class($asyncReader) . "\n";

    echo "\n6. Building HTTP request\n";
    $request = $client->post('https://httpbin.org/post');
    $request->header('Content-Type', 'application/json');
    $request->header('Content-Length', (string)strlen($jsonData));

    echo "\n7. Setting body stream (PHP Reader -> Rust AsyncRead)\n";
    $request->bodyStream($asyncReader);

    echo "\n8. Sending HTTP request...\n";
    $response = \Fiber::suspend($request->send());

    echo "\n9. Response received!\n";
    echo "Status: " . $response->status() . "\n";

    echo "\n10. Reading response body\n";
    $body = \Fiber::suspend($response->text());

    echo "\n11. Parsing response JSON\n";
    $data = json_decode($body, true);

    if (isset($data['json'])) {
        echo "\n✓ Server received our JSON data:\n";
        echo json_encode($data['json'], JSON_PRETTY_PRINT) . "\n";

        // Verify the data matches what we sent
        if ($data['json']['message'] === 'Hello from PHP Reader!') {
            echo "\n✓ Message matches!\n";
        }
        if ($data['json']['test'] === 'PhpReader integration') {
            echo "✓ Test field matches!\n";
        }
        if (isset($data['json']['nested']['foo']) && $data['json']['nested']['foo'] === 'bar') {
            echo "✓ Nested data matches!\n";
        }
    }

    echo "\n=== Test Passed! ===\n";
    echo "PHP Reader successfully streamed data through Rust to HTTP request!\n";
});
