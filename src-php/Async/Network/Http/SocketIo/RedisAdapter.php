<?php

namespace Async\Network\Http\SocketIo;

use Async\Kernel\Network\Http\SocketIo as KernelSocketIo;
use Redis\Redis;

class RedisAdapter implements AdapterInterface
{
    private ?KernelSocketIo $kernel = null;
    private Redis $pub;
    private Redis $sub;
    private string $prefix;
    private string $uid;
    private bool $subscribed = false;

    /**
     * @param Redis $pub Dedicated connection for publishing
     * @param Redis $sub Dedicated connection for subscribing
     * @param string $prefix Prefix for redis channels
     */
    public function __construct(Redis $pub, Redis $sub, string $prefix = 'socket.io')
    {
        $this->pub = $pub;
        $this->sub = $sub;
        $this->prefix = $prefix;
        $this->uid = bin2hex(random_bytes(6));
    }

    public function setKernel(KernelSocketIo $kernel): void
    {
        $this->kernel = $kernel;
        if (!$this->subscribed) {
            $this->subscribe();
            $this->subscribed = true;
        }
    }

    public function broadcast(array $packet, array $opts): void
    {
        $data = json_encode([
            'uid' => $this->uid,
            'packet' => $packet,
            'opts' => $opts
        ]);
        
        // Publish to global channel
        $this->pub->publish($this->prefix, $data);
    }

    public function add(string $id, string $room): void 
    {
        // Optional: track rooms in redis for scaling fetchSockets etc.
    }

    public function del(string $id, string $room): void {}

    public function delAll(string $id): void {}

    private function subscribe(): void
    {
        // Redis::subscribe in Rust handles the background loop via spawn_local
        $this->sub->subscribe([$this->prefix], function(string $channel, string $message) {
            if (!$this->kernel) {
                return;
            }

            $data = json_decode($message, true);
            if (!$data || !isset($data['packet']) || !isset($data['opts'])) {
                return;
            }

            // Ignore messages from self
            if (($data['uid'] ?? '') === $this->uid) {
                return;
            }

            // Call kernel->publish to distribute to local sockets
            $this->kernel->publish($data['packet'], $data['opts']);
        });
    }
}