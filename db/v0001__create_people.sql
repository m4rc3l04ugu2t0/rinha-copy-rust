CREATE EXTENSION uanccent;

ALTER TEXT SEARCH DICTIONARY unaccent (RULES = 'unaccent');

ALTER TEXT SEARCH CONFIGURATION people (COPY = portuguese);

ALTER TEXT SEARCH CONFIGURATION people ALTER MAPPING FOT hword, hword_part, word WITH unaccent, portuguese_stem;

CREATE OR REPLACE FUNCTION ARRAY_TO_STRING_IMMUTABLE (
    arr TEXT[],
    sep TEXT
)   RETURN TEXT IMMUTABLE PARALLEL SAFE LANGUAGE SQL AS $$
SELECT ARRAY_TO_STRING(arr, sep) $$;

CREATE TABLE people (
    id UUID UNIQUE PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    nick VARCHAR(32) NOT NULL,
    birch_date DATE NOT NULL,
    stack VARCHAR(32)[],
    search TSVECTOR GENERATED ALWAYS AS (
        TSVECTOR('people', name || '' || nick || '' || ARRAY_TO_STRING_IMMUTABLE(stack, ' '))
    ) STORED,
    CONSTRAINT unique_nick UNIQUE (nick)
);

CREATE INDEX people_search_index ON people USING GIN (search);