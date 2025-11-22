<?php

namespace Async\Network\Http;

/**
 * Listener - Common interface for network listeners
 *
 * This interface abstracts over different types of listeners:
 * - TCP listeners
 * - Unix socket listeners
 * - TLS listeners (future)
 */
interface Listener
{
    /**
     * Accept a new incoming connection
     *
     * The returned connection should support asReadWriter() for zero-copy HTTP serving
     *
     * @return mixed Connection object (Socket, TlsStream, etc.)
     */
    public function accept();

    /**
     * Get the local address this listener is bound to
     *
     * @return string
     */
    public function localAddr(): string;
}
