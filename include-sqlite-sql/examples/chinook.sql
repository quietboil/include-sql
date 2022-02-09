-- name: get_artist_albums?
-- Retrieves albums of the specified artist(s)
-- ## Parameters
-- param: artist_name: &str - artist matching pattern
-- ## SQL Example
-- ```sql
-- -- Ensure the record exists first:
-- INSERT INTO a_table (...) VALUES (...);
-- -- Then run the report:
-- SELECT * FROM a_view WHERE ...;
-- ```
SELECT Artist.Name AS artist_name
     , Album.Title AS album_title
  FROM Album
  JOIN Artist ON Artist.ArtistId = Album.ArtistId
 WHERE Artist.Name LIKE :artist_name
 ORDER BY 1, 2;

-- name: count_albums?
-- Returns number of albums for the specified artist(s)
-- ## Parameters
-- param: artist_name: &str - artist matching pattern
SELECT Artist.Name AS artist_name
     , Count(*)    AS num_albums
  FROM Album
  JOIN Artist ON Artist.ArtistId = Album.ArtistId
 WHERE Artist.Name LIKE :artist_name
 GROUP BY Artist.Name
 ORDER BY 2 DESC, 1;

-- name: get_customers?
-- Retrieves names of the customers from the specified state
-- that work at the specified compaies
-- ## Parameters
-- param: state: &str - state abbreviation
-- param: companies: &str - names of companies
SELECT DISTINCT LastName as last_name, FirstName as first_name
  FROM Customer
 WHERE State = :state
   AND Company IN (:companies)
 ORDER BY 1, 2;

-- name: create_new_genre!
--
-- Inserts new genre record
--
-- # Parameters
-- param: genre_id: i32 - genre ID
-- param: name: &str - name of the new genre
--
INSERT INTO Genre (GenreId, Name) VALUES (:genre_id, :name);

-- name: delete_genre->
--
-- Deletes genre record and returns name of the deleted genre
--
-- # Parameters
-- param: genre_id: i32 - genre ID
--
DELETE FROM Genre WHERE GenreId = :genre_id RETURNING Name;

-- name: begin_transaction!
-- Starts database transaction
BEGIN DEFERRED;

-- name: rollback!
-- Rolls the current transaction back
ROLLBACK;
