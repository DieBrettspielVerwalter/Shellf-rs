-- ===========================================================
-- View: Spieler_Ausleihe
-- ===========================================================
/*
Zweck:
-------
Diese View zeigt alle Ausleihen eines Spielers an:
1. Ausleihen, die der Spieler selbst ausgeliehen hat.
2. Ausleihen von Kopien, die der Spieler besitzt, aber an andere verliehen wurden.

Notwendigkeit:
---------------
- Spieler sollen ihre eigenen Ausleihen übersichtlich sehen.
- Verhindert unübersichtliche Zugriffe auf fremde Kopien.
- Ermöglicht Ausleihen-Management ohne direkte Schreibrechte auf Tabellen anderer Benutzer.

Funktionsweise:
---------------
- USER_EMAIL() wird verwendet, um den aktuellen Spieler zu identifizieren.
- Zwei Quellen werden zusammengeführt:
  a) Eigene Ausleihen (Ausleihe.Spieler_Email = USER_EMAIL())
  b) Eigene Kopien, die an andere verliehen wurden (Spielkopie.Besitzer_Email = USER_EMAIL())
*/

CREATE OR REPLACE
SQL SECURITY DEFINER
VIEW Spieler_Ausleihe AS
SELECT a.*
FROM Ausleihe a
WHERE a.Spieler_Email = USER_EMAIL()

UNION ALL

SELECT a.*
FROM Ausleihe a
JOIN Spielkopie sk ON sk.Kopie_ID = a.Kopie_ID
WHERE sk.Besitzer_Email = USER_EMAIL()
  AND a.Spieler_Email <> USER_EMAIL();

-- Eigene Partienteilnahmen
CREATE OR REPLACE
SQL SECURITY DEFINER
VIEW Spieler_Teilnahme AS
SELECT t.*
FROM Teilnahme t
WHERE t.Spieler_Email = USER_EMAIL();


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


CREATE OR REPLACE VIEW Spieler_Oeffentlich AS
    SELECT
        s.Email,
        s.Nickname,
        p.Personenvorname,
        p.Personennachname
    FROM Spieler s
             JOIN Person p ON s.Email = p.Email;


