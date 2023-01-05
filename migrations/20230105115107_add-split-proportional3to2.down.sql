ALTER TABLE expenses
DROP CONSTRAINT expenses_check1;

ALTER TABLE expenses
ALTER COLUMN split
TYPE TEXT USING split::TEXT;

DROP TYPE split;
CREATE TYPE split AS ENUM (
	'Proportional',
	'Arbitrary',
	'Evenly'
);

UPDATE expenses
SET split = 'Proportional'
WHERE split = 'Proportional2to1';

ALTER TABLE expenses
ALTER COLUMN split
TYPE split USING split::split;

ALTER TABLE expenses
ADD CONSTRAINT expenses_check1 CHECK (
	CASE split
	WHEN 'Evenly' THEN owed = paid / 2
	WHEN 'Arbitrary' THEN owed <= paid
	WHEN 'Proportional' THEN CASE payer
		WHEN 'Ale' THEN owed = paid / 3
		WHEN 'Lu' THEN owed = paid * 2 / 3
		END
	END
);
