<?php

namespace Async\Network\Http\SocketIo;

use Async\Kernel\Network\Http\SocketIo as KernelSocketIo;

interface AdapterInterface
{
    /**
     * Set the kernel instance for publishing remote events.
     */
    public function setKernel(KernelSocketIo $kernel): void;

    /**
     * Broadcast a packet to other nodes.
     */
    public function broadcast(array $packet, array $opts): void;

    /**
     * Add a socket to a room.
     */
    public function add(string $id, string $room): void;

    /**
     * Remove a socket from a room.
     */
    public function del(string $id, string $room): void;

    /**
     * Remove a socket from all rooms.
     */
    public function delAll(string $id): void;
}
