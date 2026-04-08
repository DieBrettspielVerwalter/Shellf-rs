-- ===========================================================
-- Row-Level Security Views für Spieler
-- ===========================================================
-- Spieler sollen nur ihre eigenen Kopien, Ausleihen und Partienteilnahmen sehen.
-- SQL SECURITY DEFINER sorgt dafür, dass die Prozeduren/View mit den nötigen Rechten des DEFINER laufen.
--                      Wird verwendet, da der angemeldete Nutzer normalerweise nicht ausreichend Berechtigungen hat, um in den jeweiligen Tabellen beispielsweise INSERTS auszufuehren. SQL SECURITY DEFINER erlaubt es die PROCEDURE von dem User trotdem auszufuehren und dabei INSERTS auszuloesen.
-- Dadurch können Spieler Daten einsehen und bearbeiten, die sie betreffen, ohne Zugriff auf andere Spieler zu haben.

-- ===========================================================
-- View: Spieler_Kopien_und_Ausleihen
-- ===========================================================
/*
Zweck:
-------
Diese View zeigt einem Spieler alle Spielkopien im Kontext seines Besitzes:
1. Eigene Kopien (unabhängig davon, ob sie gerade verliehen sind)
2. Kopien, die er aktuell von anderen ausgeliehen hat

Notwendigkeit:
---------------
- Übersicht über eigenen Bestand und aktuell geliehene Spiele.
- Unterstützt Planungen von Partien und Ausleihen.
- Klare Trennung von Besitz vs. Ausleihe.

Funktionsweise:
---------------
- USER_EMAIL() identifiziert den aktuellen Spieler.
- Status-Spalte zeigt "Eigene Kopie" oder "Ausgeliehen".
- CURDATE() sorgt dafür, dass nur aktuell laufende Ausleihen angezeigt werden.
*/

CREATE OR REPLACE
SQL SECURITY DEFINER
VIEW Spieler_Kopien_und_Ausleihen AS
-- Eigene Kopien
SELECT 
    sk.Kopie_ID,
    sk.Spieltitel,
    sk.Erscheinungsjahr,
    sk.Besitzer_Email,
    NULL AS Ausleiher_Email,
    NULL AS Ausleihstartdatum,
    NULL AS Ausleihenddatum,
    'Eigene Kopie' AS Status
FROM Spielkopie sk
WHERE sk.Besitzer_Email = USER_EMAIL()

UNION ALL

-- Aktuell ausgeliehene Kopien von anderen
SELECT 
    sk.Kopie_ID,
    sk.Spieltitel,
    sk.Erscheinungsjahr,
    sk.Besitzer_Email,
    a.Spieler_Email AS Ausleiher_Email,
    a.Ausleihstartdatum,
    a.Ausleihenddatum,
    'Ausgeliehen' AS Status
FROM Spielkopie sk
JOIN Ausleihe a ON sk.Kopie_ID = a.Kopie_ID
WHERE a.Spieler_Email = USER_EMAIL()
  AND (a.Ausleihenddatum IS NULL OR a.Ausleihenddatum >= CURDATE());

