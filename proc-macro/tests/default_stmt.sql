-- Test case for multiple statements where the first one can be unnamed.

-- Note that because include-sql allows comments anywhere it can only
-- detect the beginning of the next statement in a multi-statement include
-- when it encounters the `name:` marker. The only exception to this rule
-- is the very first statement.

-- This statement will get its name from the file name
select * from dual

-- name: count_tables
-- Selects the number of user tables
select count(*) from user_tables
