-- The statement in this test case uses named parameters

-- name: select_invalid_objects
-- Selects invalid user object by type
select object_name from user_objects where object_type = :object_type and status = 'INVALID'
