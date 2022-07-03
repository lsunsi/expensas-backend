CREATE TYPE label AS ENUM (
	'Market',
	'Delivery',
	'Transport',
	'Leisure',
	'Water',
	'Internet',
	'Gas',
	'Housing',
	'Electricity',
	'Furnitance'
);

ALTER TABLE expenses
ADD COLUMN label label NOT NULL,
ADD COLUMN date DATE NOT NULL,
ADD COLUMN detail TEXT;
