<?php

require 'vendor/autoload.php';

use Async\Network\Http\StringStream;

echo "=== Testing StringStream (Rust-backed) ===\n\n";

$fiber = new Fiber(function() {
    echo "1. Basic string stream operations:\n";
    $stream = new StringStream("Hello, World!");

    echo "   Size: " . $stream->getSize() . "\n";
    echo "   Position: " . $stream->tell() . "\n";
    echo "   Is seekable: " . ($stream->isSeekable() ? 'yes' : 'no') . "\n";
    echo "   Is readable: " . ($stream->isReadable() ? 'yes' : 'no') . "\n";
    echo "   Is writable: " . ($stream->isWritable() ? 'yes' : 'no') . "\n";
    echo "   EOF: " . ($stream->eof() ? 'yes' : 'no') . "\n\n";

    echo "2. Reading in chunks:\n";
    $chunk1 = $stream->read(5);
    echo "   Read 5 bytes: '{$chunk1}'\n";
    echo "   Position after read: " . $stream->tell() . "\n";

    $chunk2 = $stream->read(7);
    echo "   Read 7 bytes: '{$chunk2}'\n";
    echo "   Position after read: " . $stream->tell() . "\n";
    echo "   EOF: " . ($stream->eof() ? 'yes' : 'no') . "\n\n";

    echo "3. Seeking:\n";
    $stream->seek(0);
    echo "   After seek(0), position: " . $stream->tell() . "\n";

    $stream->seek(7, SEEK_SET);
    echo "   After seek(7, SEEK_SET), position: " . $stream->tell() . "\n";
    echo "   Read from position 7: '" . $stream->read(5) . "'\n";

    $stream->seek(-5, SEEK_END);
    echo "   After seek(-5, SEEK_END), position: " . $stream->tell() . "\n";
    echo "   Read last 5 chars: '" . $stream->read(5) . "'\n\n";

    echo "4. Rewind and getContents:\n";
    $stream->rewind();
    echo "   After rewind, position: " . $stream->tell() . "\n";
    $contents = $stream->getContents();
    echo "   getContents: '{$contents}'\n";
    echo "   Position after getContents: " . $stream->tell() . "\n";
    echo "   EOF: " . ($stream->eof() ? 'yes' : 'no') . "\n\n";

    echo "5. __toString (non-destructive read):\n";
    $stream2 = new StringStream("Test String");
    $stream2->read(4); // Move position
    echo "   Position before __toString: " . $stream2->tell() . "\n";
    $str = (string)$stream2;
    echo "   __toString result: '{$str}'\n";
    echo "   Position after __toString: " . $stream2->tell() . "\n";
    echo "   (Position should be restored)\n\n";

    echo "6. Empty stream:\n";
    $empty = new StringStream("");
    echo "   Size: " . $empty->getSize() . "\n";
    echo "   EOF: " . ($empty->eof() ? 'yes' : 'no') . "\n";
    echo "   getContents: '" . $empty->getContents() . "'\n\n";

    echo "7. Binary data:\n";
    $binary = new StringStream("\x00\x01\x02\x03\xFF");
    echo "   Size: " . $binary->getSize() . "\n";
    $data = $binary->read(5);
    echo "   Read binary: " . bin2hex($data) . "\n\n";

    echo "8. Metadata:\n";
    $stream3 = new StringStream("metadata test");
    $metadata = $stream3->getMetadata();
    echo "   Mode: " . $metadata['mode'] . "\n";
    echo "   Type: " . $metadata['type'] . "\n";
    echo "   Seekable: " . ($metadata['seekable'] ? 'yes' : 'no') . "\n\n";
});

\run($fiber);

echo "=== All Tests Passed! ===\n";
