-- ---------- Spieler ----------
-- Spieler dürfen nur eigene Daten sehen oder eintragen.
-- Direkter Zugriff auf Tabellen ist verboten, nur über Views und Stored Procedures.
-- So kann niemand fremde Kopien, Ausleihen oder Teilnahmen manipulieren.
GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Einfuegen TO spieler_role;
GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Loeschen TO spieler_role;
GRANT EXECUTE ON PROCEDURE Spieler_Ausleihe_Einfuegen TO spieler_role;
GRANT EXECUTE ON PROCEDURE Spieler_Teilnahme_Einfuegen TO spieler_role;
GRANT EXECUTE ON PROCEDURE Partie_Erstellen TO spieler_role;
GRANT EXECUTE ON PROCEDURE Spieleabend_Speichern TO spieler_role;
GRANT EXECUTE ON PROCEDURE Partie_Setze_Spieleabend TO spieler_role;
GRANT EXECUTE ON PROCEDURE Partie_Setze_Rang TO spieler_role;

GRANT EXECUTE ON FUNCTION USER_EMAIL TO gast_role;
GRANT EXECUTE ON FUNCTION USER_EMAIL TO spieler_role;
GRANT EXECUTE ON FUNCTION USER_EMAIL TO manager_role;
GRANT EXECUTE ON FUNCTION USER_EMAIL TO dba_role;


-- --------- Manager ----------
GRANT EXECUTE ON PROCEDURE SpielEinfuegenMitDetails TO manager_role;
GRANT EXECUTE ON PROCEDURE anonymize_person TO manager_role;