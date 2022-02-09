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
SELECT artist.name AS artist_name
     , album.title AS album_title
  FROM album
  JOIN artist ON artist.artist_id = album.artist_id
 WHERE artist.name LIKE :artist_name
 ORDER BY 1, 2;

-- name: count_albums?
-- Returns number of albums for the specified artist(s)
-- ## Parameters
-- param: artist_name: &str - artist matching pattern
SELECT artist.name AS artist_name
     , Count(*) AS num_albums
  FROM album
  JOIN artist ON artist.artist_id = album.artist_id
 WHERE artist.name LIKE :artist_name
 GROUP BY artist.name
 ORDER BY 2 DESC, 1;

-- name: get_customers?
-- Retrieves names of the customers from the specified state
-- that work at the specified compaies
-- ## Parameters
-- param: state: &str - state abbreviation
-- param: companies: &str - names of companies
SELECT DISTINCT last_name, first_name
  FROM customer
 WHERE state = :state
   AND company IN (:companies)
 ORDER BY 1, 2;

-- name: add_new_genre!
--
-- Inserts new genre record
--
-- # Parameters
-- param: genre_id: i32 - genre ID
-- param: name: &str - name of the new genre
--
INSERT INTO genre (genre_id, name) VALUES (:genre_id, :name);

-- name: delete_genre->
--
-- Deletes genre record and returns name of the deleted genre
--
-- # Parameters
-- param: genre_id: i32 - genre ID
--
DELETE FROM genre WHERE genre_id = :genre_id RETURNING name;
