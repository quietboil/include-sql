-- The statement in this test case uses named IN parameter(s)

-- name: select_objects_by_type
-- The object_type is used twice in this test only to check that the
-- generated SQL uses the argument list in both places and does not
-- duplicate it.
select object_name, object_type
  from user_objects
 where object_type in ( :object_types ) and generated = :generated
    or object_type in ( :object_types ) and temporary = :temporary
