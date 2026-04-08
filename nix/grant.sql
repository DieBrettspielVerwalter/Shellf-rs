USE SpieleDB;

-- Insert person
INSERT INTO Person (Email, Personenvorname, Personennachname) VALUES ('maria@spiele.de', 'Maria', 'Muster');

-- Create database user
CREATE USER IF NOT EXISTS 'maria@spiele.de'@'%' IDENTIFIED BY '';

-- Assign roles
GRANT spieler_role TO 'maria@spiele.de'@'%';
SET DEFAULT ROLE spieler_role FOR 'maria@spiele.de'@'%';

-- Insert player
INSERT INTO Spieler (Email, Nickname)
VALUES ('maria@spiele.de', 'mariaplay');

-- Grant manager role
GRANT manager_role TO 'maria@spiele.de'@'%';
SET DEFAULT ROLE manager_role FOR 'maria@spiele.de'@'%';

-- Grant procedure permissions
GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Einfuegen TO 'maria@spiele.de'@'%';
GRANT EXECUTE ON PROCEDURE Spieler_EigeneKopie_Loeschen TO 'maria@spiele.de'@'%';
GRANT EXECUTE ON PROCEDURE Spieler_Ausleihe_Einfuegen TO 'maria@spiele.de'@'%';
GRANT EXECUTE ON PROCEDURE Partie_Erstellen TO 'maria@spiele.de'@'%';

GRANT SELECT, DELETE, INSERT ON *.* TO 'maria@spiele.de'@'%';

INSERT INTO Verlag_Adresse (PLZ, Ort, Strasse, Hausnummer) VALUES ('88074', 'Ravensburg', 'Puzzleallee', '3');
INSERT INTO Verlag (Verlagsname, Verlagsadresse) VALUES ('Ravensburger', 1);
INSERT INTO Person (Email, Personenvorname, Personennachname) VALUES ('knizia@spiele.de', 'Reiner', 'Knizia');
INSERT INTO Autor (Autor_Email) VALUES ('knizia@spiele.de');
CALL SpielEinfuegenMitDetails('Wer war’s?', 1986, 'Deduction', 8, 'Ravensburger', 20, 30, 2, 4, 'knizia@spiele.de');
INSERT INTO Spielkopie (Spieltitel, Erscheinungsjahr, Besitzer_Email) VALUES ('Wer war’s?', 1986, 'maria@spiele.de');
INSERT INTO Spieleabend (Spieleabendnotizen, Gastgeber_Email, Spieleabenddatum) VALUES ('Gemütlicher Abend mit Freunden', 'maria@spiele.de', '2025-01-15');
INSERT INTO Partie (Partiedatum, Spielkopie_ID, Spieleabend_ID) VALUES ('2025-01-15', 1, 1);
INSERT INTO Teilnahme (Partie_ID, Spieler_Email, Spielerrang) VALUES (1, 'maria@spiele.de', 1);

-- Apply privilege changes
FLUSH PRIVILEGES;