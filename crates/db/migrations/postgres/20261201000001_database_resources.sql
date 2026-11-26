ALTER TABLE databases
    ADD COLUMN memory_limit_mb integer CHECK (memory_limit_mb BETWEEN 64 AND 1048576),
    ADD COLUMN cpu_limit_millis integer CHECK (cpu_limit_millis BETWEEN 50 AND 256000);
