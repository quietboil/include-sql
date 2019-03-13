--
-- name: create_table_ships
-- A test table where the fleet (of one ship) will be stored
--
CREATE TABLE ships (
  id        INTEGER PRIMARY KEY,
  name      TEXT UNIQUE,
  launched  TEXT NOT NULL
)

--
-- name: create_table_sailors
-- Table where we store sailors
--
CREATE TABLE sailors (
  id        INTEGER PRIMARY KEY,
  ship_id   INTEGER,
  name      TEXT NOT NULL,
  rank      TEXT NOT NULL,    
  FOREIGN KEY (ship_id) REFERENCES ships (id)
)

--
-- name: insert_ship
--
INSERT INTO ships (name, launched)
VALUES (:name, :year)

--
-- name: insert_sailor
--
INSERT INTO sailors (ship_id, name, rank)
VALUES (:ship, :name, :rank)

-- name: select_ship_crew
-- Selects ship crew (sailors of a given ship)
--
SELECT id, name, rank
  FROM sailors
 WHERE ship_id = :ship

--
-- name: select_ship_crew_by_rank
-- Selects sailors of a given ship that also match
-- given rank(s)
--
SELECT id, name, rank
  FROM sailors
 WHERE ship_id = :ship
   AND rank IN (:ranks)
