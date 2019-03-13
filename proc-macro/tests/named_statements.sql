-- Test case for multiple named statements.

-- name: dual_output
-- Example of a dummy select
select * from dual

-- name: user_tables_count
-- Selects the number of user tables
select count(*) from user_tables
