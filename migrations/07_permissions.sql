-- Get current database
SET @db = DATABASE();

-- ---------- Manager ----------
SET @sql = CONCAT('GRANT SELECT, INSERT, UPDATE, DELETE ON `', @db, '`.* TO manager_role');
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- ---------- Gast ----------
SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spiel TO gast_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Verlag TO gast_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Autor TO gast_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spiel_Spieleranzahl TO gast_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spiel_Spieldauer TO gast_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

-- ---------- Rollenvererbung ----------
GRANT gast_role TO spieler_role;
GRANT spieler_role TO manager_role;
GRANT manager_role TO dba_role;

-- ---------- Spieler ----------
SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spielkopie TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spieler TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Ausleihe TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT, INSERT, UPDATE, DELETE ON `', @db, '`.Partie TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spieler_Kopien TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spieler_Ausleihe TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spieler_Teilnahme TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spieler_Partie TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spieler_Spieleabend TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spieler_Kopien_und_Ausleihen TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT ON `', @db, '`.Spieler_Oeffentlich TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;

SET @sql = CONCAT('GRANT SELECT, INSERT, UPDATE, DELETE ON `', @db, '`.Teilnahme TO spieler_role');
PREPARE stmt FROM @sql; EXECUTE stmt; DEALLOCATE PREPARE stmt;