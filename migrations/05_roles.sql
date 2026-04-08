-- ###########################################################
-- SQL-Skript: Rollenkonzept für die SpieleDB (MariaDB)
-- Ziel:
--   - Zentrale Verwaltung von Benutzerrechten über Rollen
--   - Sicherstellung von Row-Level Security für Spieler
--   - Jede Rolle spiegelt die realen Aufgaben innerhalb des Vereins wider
-- Hinweise:
--   - Default-User werden für Testfälle (README) erwartet.
--   - Passwörter und Default-Accounts müssen in Produktionsumgebungen angepasst werden.
-- ###########################################################

-- ===========================================================
-- Rollen anlegen
-- ===========================================================
-- Zweck:
--   - Rollen repräsentieren Aufgabenbereiche.
--   - Rechte werden nur auf Rollenebene vergeben, nicht direkt auf Benutzer.
CREATE ROLE IF NOT EXISTS dba_role;        -- Systemadministrator, volle Rechte
CREATE ROLE IF NOT EXISTS manager_role;    -- Vereinsmanager / Redakteur
CREATE ROLE IF NOT EXISTS spieler_role;    -- Spieler, eingeschränkter Zugriff
CREATE ROLE IF NOT EXISTS gast_role;       -- Gäste, nur Leserechte

-- ===========================================================
-- Benutzer anlegen
-- ===========================================================
-- Zweck:
--   - Benutzer spiegeln reale Personen oder Funktionen wider.
-- Hinweise:
--   - Die hier angegebenen Default-Accounts dienen Test- und Beispielzwecken.
--   - Passwörter müssen im produktiven Einsatz sicher ersetzt werden.
CREATE USER IF NOT EXISTS 'dba@example.com'@'%' IDENTIFIED BY '';        -- Systemadministrator
CREATE USER IF NOT EXISTS 'manager@example.com'@'%' IDENTIFIED BY '';   -- Vereinsmanager
CREATE USER IF NOT EXISTS 'spieler@example.com'@'%' IDENTIFIED BY '';   -- Spieler
CREATE USER IF NOT EXISTS 'gast@example.com'@'%' IDENTIFIED BY '';         -- Vereinsgast

-- ===========================================================
-- Rollen den Benutzern zuweisen
-- ===========================================================
-- Zweck:
--   - Jeder Benutzer erhält die Rolle, die seinen Aufgaben entspricht.
--   - DBA erhält ADMIN OPTION, um Unterrollen zu verwalten.
GRANT dba_role TO 'dba@example.com'@'%' WITH ADMIN OPTION;
GRANT manager_role TO 'manager@example.com'@'%';
GRANT spieler_role TO 'spieler@example.com'@'%';
GRANT gast_role TO 'gast@example.com'@'%';

-- ===========================================================
-- Standardrolle setzen
-- ===========================================================
-- Zweck:
--   - Sicherstellung, dass die passende Rolle beim Login automatisch aktiv ist.
-- Hinweise:
--   - Wichtig für Konsistenz in Tests und Beispielumgebungen.
SET DEFAULT ROLE dba_role FOR 'dba@example.com'@'%';
SET DEFAULT ROLE manager_role FOR 'manager@example.com'@'%';
SET DEFAULT ROLE spieler_role FOR 'spieler@example.com'@'%';
SET DEFAULT ROLE gast_role FOR 'gast@example.com'@'%';

CREATE OR REPLACE
SQL SECURITY DEFINER
VIEW Spieler_Kopien AS
SELECT *
FROM Spielkopie
WHERE Besitzer_Email = USER_EMAIL();

-- Eigene Ausleihen
CREATE OR REPLACE
SQL SECURITY DEFINER
VIEW Spieler_Ausleihe AS
SELECT a.*
FROM Ausleihe a
WHERE a.Spieler_Email = USER_EMAIL();

-- Eigene Partienteilnahmen
CREATE OR REPLACE
SQL SECURITY DEFINER
VIEW Spieler_Teilnahme AS
SELECT t.*
FROM Teilnahme t
WHERE t.Spieler_Email = USER_EMAIL();
