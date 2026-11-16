-- SQL NULL 表示新卡尚未建立记忆状态；已有卡片同样保留为 NULL。
ALTER TABLE CARDS ADD COLUMN memory_state TEXT
    CHECK (
        CASE
            WHEN memory_state IS NULL THEN 1
            WHEN json_valid(memory_state) THEN json_type(memory_state) = 'object'
            ELSE 0
        END
    );
