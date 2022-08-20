CREATE TABLE transfers (
	id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	receiver person NOT NULL,
	sender person NOT NULL,

	amount MONEY NOT NULL,
	date DATE NOT NULL,

	confirmed_at TIMESTAMPTZ,
	refused_at TIMESTAMPTZ,

	created_at TIMESTAMPTZ NOT NULL,

	CHECK(receiver != sender),
	CHECK (num_nulls(confirmed_at, refused_at) > 0)
);
