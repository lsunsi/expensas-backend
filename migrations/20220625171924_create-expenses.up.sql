CREATE TYPE split AS ENUM (
	'Proportional',
	'Arbitrary',
	'Evenly'
);

CREATE TABLE expenses (
	id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
	creator person NOT NULL,
	payer person NOT NULL,
	split split NOT NULL,

	paid MONEY NOT NULL,
	owed MONEY NOT NULL,

	confirmed_at TIMESTAMPTZ,
	refused_at TIMESTAMPTZ,

	created_at TIMESTAMPTZ NOT NULL,

	CHECK (num_nulls(confirmed_at, refused_at) > 0),
	CHECK (
		CASE split
		WHEN 'Evenly' THEN owed = paid / 2
		WHEN 'Arbitrary' THEN owed <= paid
		WHEN 'Proportional' THEN CASE payer
			WHEN 'Ale' THEN owed = paid / 3
			WHEN 'Lu' THEN owed = paid * 2 / 3
			END
		END
	)
);
