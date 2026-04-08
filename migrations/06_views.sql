-- ===========================================================
-- View: Spieleabend relevant für den aktuell angemeldeten Spieler
-- ===========================================================
CREATE OR REPLACE
SQL SECURITY DEFINER
VIEW Spieler_Spieleabend AS
SELECT s.*
FROM Spieleabend s
WHERE 
    -- 1) Zeilen, bei denen der aktuelle Spieler der Gastgeber ist
    -- USER() liefert z.B. 'maria@spiele.de@localhost', wir müssen den Host-Teil abschneiden,
    -- damit er mit der Email-Spalte (z.B. 'maria@spiele.de') übereinstimmt.
    s.Gastgeber_Email = USER_EMAIL()

    OR
    -- 2) Zeilen, bei denen der Spieler an mindestens einer Partie dieses Spieleabends teilgenommen hat
    s.Spieleabend_ID IN (
        SELECT p.Spieleabend_ID
        FROM Partie p
        JOIN Teilnahme t 
          ON t.Partie_ID = p.Partie_ID
        WHERE t.Spieler_Email = USER_EMAIL()
    );


-- ===========================================================
-- View: Partie relevant für den aktuell angemeldeten Spieler
-- ===========================================================
CREATE OR REPLACE
SQL SECURITY DEFINER
VIEW Spieler_Partie AS
SELECT p.*
FROM Partie p
WHERE 
    -- 1) Alle Partien, bei denen der Spieler selbst teilgenommen hat
    p.Partie_ID IN (
        SELECT t.Partie_ID
        FROM Teilnahme t
        WHERE t.Spieler_Email = USER_EMAIL()
    )
    OR
    -- 2) Alle Partien, die zu einem Spieleabend gehören, bei dem der Spieler Gastgeber ist
    p.Spieleabend_ID IN (
        SELECT s.Spieleabend_ID
        FROM Spieleabend s
        WHERE s.Gastgeber_Email = USER_EMAIL()
    );

GRANT ALL PRIVILEGES ON SpieleDB.* TO dba_role;
GRANT CREATE USER ON *.* TO dba_role;
GRANT manager_role TO dba_role WITH ADMIN OPTION;
GRANT spieler_role TO dba_role WITH ADMIN OPTION;
GRANT gast_role    TO dba_role WITH ADMIN OPTION;