<?php

namespace Async\IO;

/**
 * AsyncIO interface for types that can be cast to different kernel IO types.
 *
 * This interface is implemented by all wrapper classes that wrap kernel IO objects,
 * providing zero-cost access to the underlying async IO primitives.
 */
interface TokioIO
{
    /**
     * Cast to a different IO type based on bitflags.
     *
     * @param int $type Bitflags (IO::READ | IO::WRITE | IO::SEEK | IO::BUF)
     * @return mixed Kernel IO object or wrapper
     */
    public function castTo(int $type);

    /**
     * Unwrap to get the underlying kernel IO object.
     *
     * @return object Kernel IO object (AsyncReader, AsyncWriter, etc.)
     */
    public function unwrap(): object;
}
