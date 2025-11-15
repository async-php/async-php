<?php

use Async\Kernel\Channel as KernelChannel;

require_once 'vendor/autoload.php';

$channel = new KernelChannel();
echo "Available methods:\n";
print_r(get_class_methods($channel));
