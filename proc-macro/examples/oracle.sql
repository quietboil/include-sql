--
-- name: create_test_tables
-- Creates test tables and sequences used to generate primary keys
--
DECLARE
  found INTEGER;
BEGIN
  SELECT count(*)
    INTO found
    FROM user_tables
   WHERE table_name = 'SHIPS';
  IF found = 0 THEN
    EXECUTE IMMEDIATE '
      CREATE TABLE ships (
        id        INTEGER PRIMARY KEY,
        name      VARCHAR2(50) UNIQUE,
        launched  DATE NOT NULL
      )
    ';
    EXECUTE IMMEDIATE '
      CREATE SEQUENCE ship_id_seq
    ';
  END IF;
  SELECT count(*)
    INTO found
    FROM user_tables
   WHERE table_name = 'SAILORS';
  IF found = 0 THEN
    EXECUTE IMMEDIATE '
      CREATE TABLE sailors (
        id        INTEGER PRIMARY KEY,
        ship_id   INTEGER REFERENCES ships (id) ON DELETE SET NULL,
        name      VARCHAR2(50) NOT NULL,
        rank      VARCHAR2(20) NOT NULL
      )
    ';
    EXECUTE IMMEDIATE '
      CREATE SEQUENCE sailor_id_seq
    ';
  END IF;
END;

--
-- name: insert_ship
-- Note that the returning ID will be the last "column" in the returned "row"
--
INSERT INTO ships (id, name, launched)
SELECT ship_id_seq.nextval, :name, TO_DATE(:year,'YYYY')
  FROM dual
 WHERE NOT EXISTS (
        SELECT NULL
          FROM ships
          WHERE name = :name)

--
-- name: select_ship_by_name
--
SELECT id, launched
  FROM ships
 WHERE name = :name

--
-- name: insert_sailor
--
INSERT INTO sailors (id, ship_id, name, rank)
SELECT sailor_id_seq.nextval, :ship, :name, :rank
  FROM dual
 WHERE NOT EXISTS (
        -- the logic here assumes that the fleet will not
        -- have more than one sailor with the same name 
        SELECT NULL
          FROM sailors
          WHERE name = :name)

-- name: select_ship_crew
-- Selects ships crew
--
SELECT id, name, rank
  FROM sailors
 WHERE ship_id = :ship

--
-- name: select_ship_crew_by_rank
--
SELECT id, name, rank
  FROM sailors
 WHERE ship_id = :ship
   AND rank IN (:ranks)
