<?php

namespace Async\IO;

/**
 * ReadWriteSeeker combines Reader, Writer, and Seeker interfaces
 */
interface ReadWriteSeeker extends Reader, Writer, Seeker
{
}
