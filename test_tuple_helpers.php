<?php

require_once __DIR__ . '/vendor/autoload.php';

use Async\Channel;
use Async\Kernel;

echo "=== Testing Tuple Helper Functions ===\n\n";

// 演示说明：这些是计划在应用层 (PHP wrapper) 支持的语法
// 当前测试展示了基础的 tuple 概念

echo "Future PHP API examples:\n";
echo "  [\$value, \$ok] = \$channel->pop();  // Go-style tuple return\n";
echo "  [\$result, \$error, \$ok] = \$operation->execute();  // Three-value tuple\n\n";

// 当前在 Rust 侧，我们已经有了 tuple2 和 tuple3 辅助函数
// 可以在 PHP wrapper 层封装使用

echo "Implementation note:\n";
echo "  - util::tuple2() creates [value1, value2] array\n";
echo "  - util::tuple3() creates [value1, value2, value3] array\n";
echo "  - These helpers are ready for use in Rust-side channel methods\n\n";

// 示例：未来可能的 Channel API
Kernel::run(function () {
    echo "Example: Enhanced channel API with tuple returns\n\n";

    $chan = new Channel(1);

    // 当前 API
    echo "Current API:\n";
    $result = $chan->push("test message");
    echo "  push() returns: " . ($result ? 'true' : 'false') . "\n";

    $value = $chan->pop();
    echo "  pop() returns: " . ($value ?? 'null') . "\n\n";

    // 未来可能的增强 API (需要在 PHP wrapper 实现)
    echo "Future API (to be implemented in PHP wrapper):\n";
    echo "  [\$value, \$ok] = \$channel->tryPop();\n";
    echo "  if (\$ok) {\n";
    echo "      echo \"Received: \$value\";\n";
    echo "  } else {\n";
    echo "      echo \"Channel empty or closed\";\n";
    echo "  }\n\n";

    $chan->close();
});

echo "=== Tuple helpers are ready in util.rs! ===\n";
echo "Next step: Implement PHP wrapper methods using tuple2()/tuple3()\n";
