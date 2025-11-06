<?php

namespace Async;

use Async\IO\Wrapper\ReaderWrapper;
use Async\IO\Wrapper\WriterWrapper;
use Async\IO\Wrapper\SeekerWrapper;
use Async\IO\Wrapper\BufReaderWrapper;
use Async\Kernel\IO\AsyncReader;
use Async\Kernel\IO\AsyncWriter;
use Async\Kernel\IO\AsyncSeeker;
use Async\Kernel\IO\AsyncBufReader;

/**
 * IO provides type conversion methods between Kernel IO types and Wrapper types
 */
class IO
{
    /**
     * Wrap AsyncReader into a ReaderWrapper
     *
     * @param AsyncReader $reader The kernel async reader
     * @return ReaderWrapper Reader interface implementation
     */
    public static function wrapReader(AsyncReader $reader): ReaderWrapper
    {
        return new ReaderWrapper($reader);
    }

    /**
     * Wrap AsyncWriter into a WriterWrapper
     *
     * @param AsyncWriter $writer The kernel async writer
     * @return WriterWrapper Writer interface implementation
     */
    public static function wrapWriter(AsyncWriter $writer): WriterWrapper
    {
        return new WriterWrapper($writer);
    }

    /**
     * Wrap AsyncSeeker into a SeekerWrapper
     *
     * @param AsyncSeeker $seeker The kernel async seeker
     * @return SeekerWrapper Seeker interface implementation
     */
    public static function wrapSeeker(AsyncSeeker $seeker): SeekerWrapper
    {
        return new SeekerWrapper($seeker);
    }

    /**
     * Wrap AsyncBufReader into a BufReaderWrapper
     *
     * @param AsyncBufReader $reader The kernel async buffered reader
     * @return BufReaderWrapper BufReader interface implementation
     */
    public static function wrapBufReader(AsyncBufReader $reader): BufReaderWrapper
    {
        return new BufReaderWrapper($reader);
    }
}
