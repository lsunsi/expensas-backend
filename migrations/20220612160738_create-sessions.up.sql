CREATE TYPE person AS ENUM ('Ale', 'Lu');

CREATE TABLE sessions (
	id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	who person NOT NULL,
	confirmed_at TIMESTAMPTZ,
	converted_at TIMESTAMPTZ,
	refused_at TIMESTAMPTZ,
	created_at TIMESTAMPTZ NOT NULL,
	CHECK (confirmed_at IS NOT NULL OR converted_at IS NULL),
	CHECK (num_nulls(confirmed_at, refused_at) > 0)
);
