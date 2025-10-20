<?php

namespace Async\Http;

use Async\Network\TcpServer;
use Async\Kernel;
use HttpParser;
use HttpReq;

class Server
{
    private TcpServer $tcp;
    private $handler;

    public function __construct(string $host, int $port)
    {
        $this->tcp = new TcpServer($host, $port);
    }

    public function handle(callable $handler): void
    {
        $this->handler = $handler;
        
        while (true) {
            try {
                $socket = $this->tcp->accept();
                Kernel::spawn(function () use ($socket) {
                    try {
                        // Simple Read Loop
                        // In real world, we need to buffer until \r\n\r\n
                        $buffer = '';
                        while (true) {
                            $chunk = $socket->read(8192);
                            if ($chunk === '') break;
                            
                            $buffer .= $chunk;
                            
                            // Try parse
                            $req = HttpParser::parseRequest($buffer);
                            if ($req) {
                                // Handover to user handler
                                $res = ($this->handler)($req);
                                
                                // Send Response
                                $response = "HTTP/1.1 200 OK\r\nContent-Length: " . strlen($res) . "\r\n\r\n" . $res;
                                $socket->write($response);
                                break; // Close for HTTP/1.0 style
                            }
                            
                            if (strlen($buffer) > 16384) break; // Flood protection
                        }
                    } finally {
                        $socket->close();
                    }
                });
            } catch (
Exception $e) {
                // Ignore accept errors
            }
        }
    }
}
