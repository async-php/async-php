<?php

namespace PDO;

class PDOException extends \Exception
{
    /**
     * @var array{0:string,1:int|string|null,2:string|null}|null
     */
    public ?array $errorInfo = null;
}

