--
-- name: create_table_ships
-- A test table where the fleet (of one ship) will be stored
--
CREATE TABLE IF NOT EXISTS ships (
  id        SERIAL PRIMARY KEY,
  name      TEXT UNIQUE,
  launched  TEXT NOT NULL
)

--
-- name: create_table_sailors
-- Table where we store sailors
--
CREATE TABLE IF NOT EXISTS sailors (
  id        SERIAL PRIMARY KEY,
  ship_id   INTEGER REFERENCES ships (id),
  name      TEXT NOT NULL,
  rank      TEXT NOT NULL
)

--
-- name: insert_ship
-- Its return is the same projection as in the select_ship_by_name
-- even though launched is not really used in the demo as the result
-- serves the same purpose - get the existing or the just inserted
-- ship ID
--
INSERT INTO ships (name, launched)
VALUES (:name, :year)
RETURNING id, launched

--
-- name: select_ship_by_name
--
SELECT id, launched
  FROM ships
 WHERE name = :name

--
-- name: insert_sailor
--
INSERT INTO sailors (ship_id, name, rank)
VALUES (:ship, :name, :rank)

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
