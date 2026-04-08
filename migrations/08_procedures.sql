CREATE OR REPLACE PROCEDURE Spieler_EigeneKopie_Einfuegen(
    IN p_Spieltitel VARCHAR(100),
    IN p_Erscheinungsjahr YEAR(4)
)
SQL SECURITY DEFINER
BEGIN
    DECLARE v_email VARCHAR(100);

    -- Aktuellen Benutzer ermitteln
    SET v_email = USER_EMAIL();

    -- Eigene Spielkopie einfügen und Benutzer als Besitzer eintragen
    INSERT INTO Spielkopie (Spieltitel, Erscheinungsjahr, Besitzer_Email)
    VALUES (p_Spieltitel, p_Erscheinungsjahr, v_email);
END;


CREATE OR REPLACE PROCEDURE Spieler_EigeneKopie_Loeschen(
    IN p_id INT
)
SQL SECURITY DEFINER
BEGIN
    DECLARE v_email VARCHAR(100);
    DECLARE v_count INT;

    SET v_email = USER_EMAIL();

    -- Check ownership
    SELECT COUNT(*) INTO v_count
    FROM Spielkopie
    WHERE Kopie_ID = p_id
      AND Besitzer_Email = v_email;

    IF v_count = 0 THEN
        SIGNAL SQLSTATE '45000'
        SET MESSAGE_TEXT = 'Zugriff verweigert: Nur eigene Kopien erlaubt';
    END IF;

    -- Delete if authorized
    DELETE FROM Spielkopie
    WHERE Kopie_ID = p_id
      AND Besitzer_Email = v_email;
END;


CREATE OR REPLACE PROCEDURE Spieler_Ausleihe_Einfuegen(
    IN p_Kopie_ID INT,
    IN p_Spieler_Email VARCHAR(100),
    IN p_Ausleihstart DATE,
    IN p_Ausleihend DATE
)
SQL SECURITY DEFINER
BEGIN
    DECLARE v_email VARCHAR(100);

    -- Aktuellen Benutzer ermitteln
    SET v_email = USER_EMAIL();

    -- Prüfen, ob der Benutzer Eigentümer der Kopie ist
    IF EXISTS (
        SELECT 1 
        FROM Spielkopie 
        WHERE Kopie_ID = p_Kopie_ID 
          AND Besitzer_Email = v_email
    ) THEN
        -- Neue Ausleihe eintragen
        INSERT INTO Ausleihe (Kopie_ID, Spieler_Email, Ausleihstartdatum, Ausleihenddatum)
        VALUES (p_Kopie_ID, p_Spieler_Email, p_Ausleihstart, p_Ausleihend);
    ELSE
        -- Abbruch, wenn kein Eigentum besteht
        SIGNAL SQLSTATE '45000' 
            SET MESSAGE_TEXT = 'Zugriff verweigert: Nur eigene Kopien erlaubt';
    END IF;
END;


CREATE OR REPLACE PROCEDURE Spieler_Teilnahme_Einfuegen(
    IN p_Partie_ID INT
)
SQL SECURITY DEFINER
BEGIN
    DECLARE v_email VARCHAR(100);

    -- Aktuellen Benutzer ermitteln
    SET v_email = USER_EMAIL();

    -- Prüfen, ob die Partie existiert
    IF EXISTS (SELECT 1 FROM Partie WHERE Partie_ID = p_Partie_ID) THEN
        -- Teilnahme eintragen
        INSERT INTO Teilnahme (Partie_ID, Spieler_Email)
        VALUES (p_Partie_ID, v_email);
    ELSE
        -- Abbruch, wenn die Partie nicht existiert
        SIGNAL SQLSTATE '45000'
            SET MESSAGE_TEXT = 'Partie existiert nicht';
    END IF;
END;


CREATE OR REPLACE PROCEDURE Partie_Erstellen(
    IN p_Kopie_ID INT,
    IN p_Partiedatum DATE
)
SQL SECURITY DEFINER
BEGIN
    DECLARE v_email VARCHAR(100);         -- Aktueller Benutzer, bestimmt Schreibrechte
    DECLARE v_partie_id INT;             -- Automatisch generierte Partie-ID

    -- Den aktuellen Benutzer (Spieler) ermitteln
    SET v_email = USER_EMAIL();

    -- Überprüfen, ob der Spieler eine Spielkopie besitzt oder aktuell ausgeliehen hat
    IF EXISTS (
        SELECT 1
        FROM Spielkopie sk
        WHERE sk.Kopie_ID = p_Kopie_ID
        AND (sk.Besitzer_Email = v_email OR
             EXISTS (
                 SELECT 1
                 FROM Ausleihe a
                 WHERE a.Kopie_ID = sk.Kopie_ID
                 AND a.Spieler_Email = v_email
		 AND a.Ausleihstartdatum <= p_Partiedatum
                 AND (a.Ausleihenddatum IS NULL OR a.Ausleihenddatum >= p_Partiedatum)
             ))
    ) THEN
        -- Berechtigung liegt vor, Partie erstellen
        INSERT INTO Partie (Partiedatum, Spielkopie_ID)
        VALUES (p_Partiedatum, p_Kopie_ID);

        -- Die automatisch generierte Partie_ID abrufen
        SET v_partie_id = LAST_INSERT_ID();

        -- Spieler zur Partie hinzufügen
        CALL Spieler_Teilnahme_Einfuegen(v_partie_id);

    ELSE
        -- Keine Berechtigung → Fehler auslösen
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'Ersteller hat keine Berechtigung für die Spielkopie';
    END IF;
END;


CREATE OR REPLACE PROCEDURE Partie_Setze_Spieleabend(
    IN p_Partie_ID INT,
    IN p_Spieleabend_ID INT
)
SQL SECURITY DEFINER
BEGIN
    DECLARE v_email VARCHAR(100);          -- Aktueller Benutzer
    DECLARE v_is_root BOOLEAN DEFAULT FALSE; -- Root-Status (Standard FALSE, ggf. anpassen)
    DECLARE v_is_dba BOOLEAN DEFAULT FALSE;  -- DBA-Status (Standard FALSE, ggf. anpassen)
    DECLARE v_teilnehmer_count INT;        -- Prüft, ob Benutzer Teilnehmer ist
    DECLARE v_msg TEXT;                    -- Meldung für SIGNAL

    -- Aktuellen Benutzer ermitteln
    SET v_email = USER_EMAIL();

    -- root erkennen
    IF v_email = 'root' THEN
        SET v_is_root = TRUE;
    END IF;

    -- dba_role erkennen
    IF FIND_IN_SET('dba_role', CURRENT_ROLE()) THEN
        SET v_is_dba = TRUE;
    END IF;

    -- Prüfen, ob der Benutzer Teilnehmer der Partie ist
    SELECT COUNT(*) INTO v_teilnehmer_count
    FROM Teilnahme
    WHERE Partie_ID = p_Partie_ID
      AND Spieler_Email = v_email;

    -- ------------------------------------------------------------------
    -- Berechtigungsprüfung
    -- ------------------------------------------------------------------
    IF v_teilnehmer_count = 0 THEN
        IF v_is_root THEN
            -- root → alles ok, keine Warnung
            SET @dummy = 0;

        ELSEIF v_is_dba THEN
            -- dba_role → Warnung, aber erlaubt
            SET v_msg = CONCAT(
                'Warnung (DBA): Spieleabend_ID wird gesetzt, obwohl Benutzer kein Teilnehmer ist. ',
                'Partie_ID=', p_Partie_ID,
                ', Spieleabend_ID=', p_Spieleabend_ID
            );
            SIGNAL SQLSTATE '01000' SET MESSAGE_TEXT = v_msg;

        ELSE
            -- normale Benutzer → harter Fehler
            SET v_msg = 'Ungültig: Nur Teilnehmer, root oder DBA dürfen das Spieleabend-Feld ändern.';
            SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
        END IF;
    END IF;

    -- ------------------------------------------------------------------
    -- Spieleabend_ID setzen oder aktualisieren
    -- ------------------------------------------------------------------
    UPDATE Partie
    SET Spieleabend_ID = p_Spieleabend_ID
    WHERE Partie_ID = p_Partie_ID;

END;


CREATE OR REPLACE PROCEDURE Partie_Setze_Rang(
    IN p_Partie_ID INT,
    IN p_Rang INT
)
SQL SECURITY DEFINER
BEGIN
    DECLARE v_email VARCHAR(100);            -- aktueller Benutzer
    DECLARE v_is_root BOOLEAN DEFAULT FALSE; -- Root-Status (Standard FALSE, ggf. anpassen)
    DECLARE v_is_dba BOOLEAN DEFAULT FALSE;  -- DBA-Status (Standard FALSE, ggf. anpassen)
    DECLARE v_teilnehmer_count INT;          -- Prüft, ob Benutzer Teilnehmer ist
    DECLARE v_msg TEXT;                      -- Meldung für SIGNAL

    -- Aktuellen Benutzer ermitteln
    SET v_email = USER_EMAIL();

    -- root erkennen
    IF v_email = 'root' THEN
        SET v_is_root = TRUE;
    END IF;

    -- dba_role erkennen
    IF FIND_IN_SET('dba_role', CURRENT_ROLE()) THEN
        SET v_is_dba = TRUE;
    END IF;

    -- Prüfen, ob der Benutzer Teilnehmer der Partie ist
    SELECT COUNT(*) INTO v_teilnehmer_count
    FROM Teilnahme
    WHERE Partie_ID = p_Partie_ID
      AND Spieler_Email = v_email;

    -- ------------------------------------------------------------------
    -- Berechtigungsprüfung
    -- ------------------------------------------------------------------
    IF v_teilnehmer_count = 0 THEN
        IF v_is_root THEN
            -- root → alles ok, keine Warnung
            SET @dummy = 0;

        ELSEIF v_is_dba THEN
            -- dba_role → Warnung, aber erlaubt
            SET v_msg = CONCAT(
                'Warnung (DBA): Spielerrang wird gesetzt, obwohl Benutzer nicht der Spieler ist. ',
                'Partie_ID=', p_Partie_ID,
                ', Rang=', p_Rang
            );
            SIGNAL SQLSTATE '01000' SET MESSAGE_TEXT = v_msg;

        ELSE
            -- normale Benutzer → harter Fehler
            SET v_msg = 'Ungültig: Nur der Spieler selbst, root oder DBA dürfen den Rang ändern.';
            SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
        END IF;
    END IF;

    -- ------------------------------------------------------------------
    -- Spielerrang setzen oder aktualisieren
    -- ------------------------------------------------------------------
    UPDATE Teilnahme
    SET Spielerrang = p_Rang
    WHERE Partie_ID = p_Partie_ID
      AND Spieler_Email = v_email;

END;


CREATE OR REPLACE PROCEDURE SpielEinfuegenMitDetails(
    IN p_Spieltitel VARCHAR(100),
    IN p_Erscheinungsjahr YEAR(4),
    IN p_Kategorie VARCHAR(50),
    IN p_Altersempfehlung INT,
    IN p_Verlagsname VARCHAR(100),
    IN p_Spieldauer_min INT,
    IN p_Spieldauer_max INT,
    IN p_Spieleranzahl_min INT,
    IN p_Spieleranzahl_max INT,
    IN p_Autor_Email VARCHAR(100)
)
SQL SECURITY DEFINER
BEGIN
    -- 1. Sicherstellen, dass die Person existiert (verhindert FK-Fehler in Autor)
    INSERT IGNORE INTO Person (Email, Personenvorname, Personennachname, Personname)
    VALUES (p_Autor_Email, 'Unbekannter', 'Autor', 'Unbekannter Autor');

    -- 2. Sicherstellen, dass die Person als Autor markiert ist (verhindert FK-Fehler in Designs)
    INSERT IGNORE INTO Autor (Autor_Email)
    VALUES (p_Autor_Email);

    -- 3. Spiel in die Haupttabelle einfügen
INSERT INTO Spiel (Spieltitel, Erscheinungsjahr, Kategorie, Altersempfehlung, Verlagsname, Spieldauer_Durchschnitt)
VALUES (p_Spieltitel, p_Erscheinungsjahr, p_Kategorie, p_Altersempfehlung, p_Verlagsname, (p_Spieldauer_min + p_Spieldauer_max)/2)
    ON DUPLICATE KEY UPDATE Kategorie = p_Kategorie;

-- 4. Details in die Nebentabellen einfügen
INSERT IGNORE INTO Spiel_Spieldauer (Spieltitel, Erscheinungsjahr, Spieldauer_min, Spieldauer_max)
    VALUES (p_Spieltitel, p_Erscheinungsjahr, p_Spieldauer_min, p_Spieldauer_max);

    INSERT IGNORE INTO Spiel_Spieleranzahl (Spieltitel, Erscheinungsjahr, Spieleranzahl_min, Spieleranzahl_max)
    VALUES (p_Spieltitel, p_Erscheinungsjahr, p_Spieleranzahl_min, p_Spieleranzahl_max);

    -- 5. Jetzt erst die Verknüpfung in 'Designs' erstellen
    INSERT IGNORE INTO Designs (Spieltitel, Erscheinungsjahr, Autor_Email)
    VALUES (p_Spieltitel, p_Erscheinungsjahr, p_Autor_Email);
END;


CREATE OR REPLACE PROCEDURE Spieleabend_Speichern(
    IN p_Spieleabend_ID INT,       -- NULL für neuen Abend
    IN p_Spieleabenddatum DATE,
    IN p_Notizen TEXT
)
SQL SECURITY DEFINER
BEGIN
    DECLARE v_email VARCHAR(100);
    DECLARE v_count INT;

    -- Aktuellen Benutzer ermitteln
    SET v_email = USER_EMAIL();

    IF p_Spieleabend_ID IS NULL OR p_Spieleabend_ID = 0 THEN
        -- --------------------------------------------------------------
        -- Neuer Spieleabend anlegen
        -- --------------------------------------------------------------
        INSERT INTO Spieleabend (Spieleabenddatum, Spieleabendnotizen, Gastgeber_Email)
        VALUES (p_Spieleabenddatum, p_Notizen, v_email);

    ELSE
        -- --------------------------------------------------------------
        -- Bestehenden Spieleabend bearbeiten
        -- --------------------------------------------------------------
        -- Prüfen, ob der aktuelle Benutzer Gastgeber ist
        SELECT COUNT(*) INTO v_count
        FROM Spieleabend
        WHERE Spieleabend_ID = p_Spieleabend_ID
          AND Gastgeber_Email = v_email;

        IF v_count = 0 THEN
            SIGNAL SQLSTATE '45000'
                SET MESSAGE_TEXT = 'Nur der Gastgeber darf diesen Spieleabend bearbeiten.';
        ELSE
            -- Notizen und Datum überschreiben
            UPDATE Spieleabend
            SET Spieleabenddatum = p_Spieleabenddatum,
                Spieleabendnotizen = p_Notizen
            WHERE Spieleabend_ID = p_Spieleabend_ID;
        END IF;
    END IF;

END;