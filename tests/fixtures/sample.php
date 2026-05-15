<?php

namespace Demo\Core;

interface LoggerInterface
{
    public function log(string $message): void;
}

trait WithContext
{
    private function writeInternal(): void {}
}

class Logger implements LoggerInterface
{
    public const VERSION = '1.0';
    public static $shared;

    public function log(string $message): void {}
}

function create_logger(): Logger
{
    return new Logger();
}
