<?php

require_once __DIR__ . '/../vendor/autoload.php';

use Async\Network\Http\Server;
use Async\Network\Http\Request;
use Async\Network\Http\Response;
use Async\IO\Reader;

/**
 * Example Reader implementation for streaming large files
 */
class FileReader implements Reader
{
    private $handle;
    private int $chunkSize;

    public function __construct(string $path, int $chunkSize = 8192)
    {
        $this->handle = fopen($path, 'r');
        if ($this->handle === false) {
            throw new RuntimeException("Failed to open file: $path");
        }
        $this->chunkSize = $chunkSize;
    }

    public function read(int $length): ?string
    {
        if (feof($this->handle)) {
            return null;
        }

        $data = fread($this->handle, min($length, $this->chunkSize));
        return $data === false ? null : ($data === '' ? null : $data);
    }

    public function __destruct()
    {
        if (is_resource($this->handle)) {
            fclose($this->handle);
        }
    }
}

/**
 * Example Reader for generating data on-the-fly
 */
class GeneratorReader implements Reader
{
    private int $count = 0;
    private int $max;

    public function __construct(int $max = 100)
    {
        $this->max = $max;
    }

    public function read(int $length): ?string
    {
        if ($this->count >= $this->max) {
            return null;
        }

        $line = "Line {$this->count}: " . str_repeat('x', 50) . "\n";
        $this->count++;

        return $line;
    }
}

// Start server
Server::create('127.0.0.1:8080', function (Request $request): Response {
    $path = parse_url($request->getUri(), PHP_URL_PATH);

    return match ($path) {
        // Stream a file
        '/file' => (new Response(200))
            ->withHeader('Content-Type', 'application/octet-stream')
            ->withStreamBody(new FileReader(__FILE__)),

        // Stream generated content
        '/stream' => (new Response(200))
            ->withHeader('Content-Type', 'text/plain')
            ->withStreamBody(new GeneratorReader(1000)),

        // Handle POST with body
        '/echo' => (function () use ($request): Response {
            $body = $request->getBody();
            if ($body === null) {
                return Response::json(['error' => 'No body'], 400);
            }

            // Read all body content
            $content = $body->readAll();

            return Response::json([
                'echoed' => $content,
                'length' => strlen($content),
                'method' => $request->getMethod(),
            ]);
        })(),

        // Parse JSON body
        '/json' => (function () use ($request): Response {
            try {
                $data = $request->getBodyAsJson();
                return Response::json([
                    'received' => $data,
                    'processed' => true,
                ]);
            } catch (Exception $e) {
                return Response::json([
                    'error' => $e->getMessage(),
                ], 400);
            }
        })(),

        default => Response::html('<h1>Streaming Examples</h1>
            <ul>
                <li><a href="/file">Stream this file</a></li>
                <li><a href="/stream">Stream generated content</a></li>
                <li>POST to /echo to echo body</li>
                <li>POST JSON to /json to parse</li>
            </ul>'),
    };
});
