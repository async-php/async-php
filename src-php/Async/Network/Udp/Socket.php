<?php

namespace Async\Network\Udp;

use Async\IO;
use Async\IO\TokioIO;
use Async\Kernel\Network\UdpSocket as KernelUdpSocket;
use Fiber;

class Socket implements TokioIO
{
    private KernelUdpSocket $inner;

    public function __construct(KernelUdpSocket $inner)
    {
        $this->inner = $inner;
    }

    public static function bind(string $addr): self
    {
        $future = KernelUdpSocket::bind($addr);
        $kernelSocket = Fiber::suspend($future);
        if (!$kernelSocket) {
            throw new \RuntimeException("Failed to bind to $addr");
        }
        return new self($kernelSocket);
    }

    public function recvFrom(int $length = 65535): array|false
    {
        // Rust returns [data, addr]
        $future = $this->inner->recvFrom($length);
        return Fiber::suspend($future);
    }

    public function sendTo(string $data, string $addr): int|false
    {
        $future = $this->inner->sendTo($data, $addr);
        return Fiber::suspend($future);
    }

    /**
     * Peek at incoming data without removing it from the buffer
     * @return array|false [data, sender_address] or false on error
     */
    public function peekFrom(int $length = 65535): array|false
    {
        $future = $this->inner->peek_from($length);
        return Fiber::suspend($future);
    }

    /**
     * Connect this UDP socket to a remote address
     * After connecting, you can use send() and recv() without specifying address
     */
    public function connect(string $addr): bool
    {
        $future = $this->inner->connect($addr);
        return (bool)Fiber::suspend($future);
    }

    /**
     * Receive a datagram from the connected remote address
     */
    public function recv(int $length = 65535): ?string
    {
        $future = $this->inner->recv($length);
        $result = Fiber::suspend($future);
        return $result ?: null;
    }

    /**
     * Send a datagram to the connected remote address
     */
    public function send(string $data): int|false
    {
        $future = $this->inner->send($data);
        return Fiber::suspend($future);
    }

    /**
     * Get the local address this socket is bound to
     */
    public function localAddr(): string
    {
        return $this->inner->local_addr();
    }

    /**
     * Get the remote address this socket is connected to (if connected)
     */
    public function peerAddr(): string
    {
        return $this->inner->peer_addr();
    }

    /**
     * Get the value of the SO_BROADCAST option
     */
    public function broadcast(): bool
    {
        return $this->inner->broadcast();
    }

    /**
     * Set the value of the SO_BROADCAST option
     */
    public function setBroadcast(bool $broadcast): bool
    {
        return $this->inner->set_broadcast($broadcast);
    }

    /**
     * Get the value of the IP_TTL option
     */
    public function ttl(): int
    {
        return $this->inner->ttl();
    }

    /**
     * Set the value of the IP_TTL option
     */
    public function setTtl(int $ttl): bool
    {
        return $this->inner->set_ttl($ttl);
    }

    /**
     * Join an IPv4 multicast group
     * @param string $multicastAddr Multicast group address
     * @param string $interfaceAddr Local interface address
     */
    public function joinMulticastV4(string $multicastAddr, string $interfaceAddr): bool
    {
        return $this->inner->join_multicast_v4($multicastAddr, $interfaceAddr);
    }

    /**
     * Leave an IPv4 multicast group
     * @param string $multicastAddr Multicast group address
     * @param string $interfaceAddr Local interface address
     */
    public function leaveMulticastV4(string $multicastAddr, string $interfaceAddr): bool
    {
        return $this->inner->leave_multicast_v4($multicastAddr, $interfaceAddr);
    }

    /**
     * Join an IPv6 multicast group
     * @param string $multicastAddr Multicast group address
     * @param int $interfaceIndex Local interface index
     */
    public function joinMulticastV6(string $multicastAddr, int $interfaceIndex): bool
    {
        return $this->inner->join_multicast_v6($multicastAddr, $interfaceIndex);
    }

    /**
     * Leave an IPv6 multicast group
     * @param string $multicastAddr Multicast group address
     * @param int $interfaceIndex Local interface index
     */
    public function leaveMulticastV6(string $multicastAddr, int $interfaceIndex): bool
    {
        return $this->inner->leave_multicast_v6($multicastAddr, $interfaceIndex);
    }

    /**
     * Get the value of the IP_MULTICAST_LOOP option (IPv4)
     */
    public function multicastLoopV4(): bool
    {
        return $this->inner->multicast_loop_v4();
    }

    /**
     * Set the value of the IP_MULTICAST_LOOP option (IPv4)
     */
    public function setMulticastLoopV4(bool $enabled): bool
    {
        return $this->inner->set_multicast_loop_v4($enabled);
    }

    /**
     * Get the value of the IP_MULTICAST_TTL option (IPv4)
     */
    public function multicastTtlV4(): int
    {
        return $this->inner->multicast_ttl_v4();
    }

    /**
     * Set the value of the IP_MULTICAST_TTL option (IPv4)
     */
    public function setMulticastTtlV4(int $ttl): bool
    {
        return $this->inner->set_multicast_ttl_v4($ttl);
    }

    /**
     * Get the value of the IPV6_MULTICAST_LOOP option
     */
    public function multicastLoopV6(): bool
    {
        return $this->inner->multicast_loop_v6();
    }

    /**
     * Set the value of the IPV6_MULTICAST_LOOP option
     */
    public function setMulticastLoopV6(bool $enabled): bool
    {
        return $this->inner->set_multicast_loop_v6($enabled);
    }

    /**
     * Get the underlying kernel UdpSocket
     * @internal
     */
    public function unwrap(): KernelUdpSocket
    {
        return $this->inner;
    }

    /**
     * Cast to an IO wrapper based on bitflags.
     *
     * @param int $type Bitflags (IO::READ | IO::WRITE | IO::SEEK | IO::BUF)
     * @return mixed Wrapper IO object
     */
    public function castTo(int $type)
    {
        $kernelIo = $this->inner->castTo($type);
        return IO::kernelToWrapper($kernelIo);
    }
}