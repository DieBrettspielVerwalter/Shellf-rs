-- ===========================================================
-- Funktion: USER_EMAIL()
-- ===========================================================
-- Zweck:
-- Diese Funktion extrahiert die reine Benutzerkennung (ohne Host) aus USER().
-- Sie wird als Hilfsfunktion für Row-Level Security und Berechtigungsprüfungen
-- in Triggers, Procedures und Views verwendet.
-- -------------------------------------------------------------------
-- Motivation:
-- - Mehrere Datenbankprozeduren benötigen die "reine" Benutzerkennung.
-- - Vermeidet wiederholte Parsing-Logik in jedem Trigger oder Procedure.
-- - Erleichtert die Implementierung von Rollen- und Berechtigungsprüfungen
--   (z.B. Root, DBA, Standard-User, Dummy-Test-User).
-- -------------------------------------------------------------------


CREATE FUNCTION USER_EMAIL()
RETURNS VARCHAR(255)
DETERMINISTIC
BEGIN
    DECLARE v_email VARCHAR(255); -- Zwischenspeicher für die vollständige Benutzerkennung inkl. Host
    DECLARE v_host  VARCHAR(255); -- Zwischenspeicher für den Hostteil der Benutzerkennung

    -- 1. Gesamten Benutzer inkl. Host abrufen (Format: 'user@host')
    SET v_email = USER();

    -- 2. Hostteil extrahieren (alles nach '@')
    SET v_host = SUBSTRING_INDEX(v_email, '@', -1);

    -- 3. Hostteil vom ursprünglichen String entfernen, übrig bleibt reine Benutzerkennung
    SET v_email = LEFT(v_email, LENGTH(v_email) - LENGTH(v_host) - 1);

    -- 4. Ergebnis zurückgeben
    RETURN v_email;
END;




-- Dieser Zähler dient dazu, für gelöschte E-Mail-Adressen einen
-- eindeutigen, fortlaufenden Platzhalter (z. B. "geloescht1@example.com", "geloescht2@example.com", ...)
-- zu erzeugen.
--
-- Hintergrund:
-- Beim DSGVO-konformen Löschen von Personen werden personenbezogene
-- Daten – insbesondere die E-Mail-Adresse – irreversibel entfernt.
-- Um dennoch referenzielle Integrität oder technische Anforderungen
-- (z. B. UNIQUE-Constraints) zu erfüllen, wird anstelle der echten
-- E-Mail eine generierte, nicht rückführbare Dummy-Adresse verwendet.
--
-- Der hier gespeicherte Counter stellt sicher, dass jede generierte
-- „gelöschte“ E-Mail-Adresse eindeutig bleibt.
CREATE TABLE geloescht_counter (
    -- Da die Tabelle nur einen einzigen Zähler speichert,
    -- dient dieses Feld dazu, den einzigen Datensatz eindeutig zu identifizieren.
    id INT PRIMARY KEY DEFAULT 1,
    counter INT NOT NULL
);

-- Initialwert
INSERT INTO geloescht_counter (id, counter) VALUES (1, 1);


-- Trigger zum automatischen Setzen eines vollständigen Personnamens
-- beim Einfügen neuer Datensätze.
--
-- Hintergrund:
-- International existieren unterschiedliche Konventionen, wie Vor- und Nachname
-- zu einem Anzeigenamen zusammengesetzt werden.
-- In vielen westlichen Ländern (z. B. Deutschland, Österreich, USA)
-- lautet die übliche Reihenfolge: "Vorname Nachname".
--
-- In anderen Ländern hingegen – z. B. Japan, China oder Korea – ist die
-- Reihenfolge umgekehrt: "Nachname Vorname".
--
-- Da der überwiegende Anteil der Nutzer (im typischen westlich geprägten
-- Softwareeinsatzumfeld) die Form *Vorname Nachname* erwartet, wird diese
-- Standarddarstellung automatisch erzeugt, **wenn der Client keine eigene
-- Angabe liefert**.
--
-- Gleichzeitig bleibt die Möglichkeit bestehen, den Personennamen explizit
-- zu setzen. Dadurch können auch internationale Nutzer oder Spezialfälle
-- korrekt abgebildet werden, ohne dass für die Mehrheit zusätzlicher Aufwand
-- entsteht.

CREATE TRIGGER trg_person_insert
BEFORE INSERT ON Person
FOR EACH ROW
BEGIN
    -- Nur setzen, wenn Personname NULL oder leer ist
    IF NEW.Personname IS NULL OR NEW.Personname = '' THEN
        SET NEW.Personname = CONCAT(NEW.Personenvorname, ' ', NEW.Personennachname);
    END IF;
END;



-- Trigger zum automatischen Setzen bzw. Nachführen des Personnamens
-- bei Aktualisierungen.
--
-- Begründung:
-- Der Mechanismus aus dem INSERT-Trigger muss *auch beim UPDATE* greifen,
-- weil sich Vor- oder Nachname nachträglich ändern können.
-- Ohne diesen Trigger könnte der Personname nach einer solchen Änderung
-- leer, veraltet oder technisch inkonsistent bleiben.
--
-- Gleichzeitig wird – gemäß der oben beschriebenen Logik – ein vom Client
-- bewusst gesetzter Personname (z. B. für internationale Formatvarianten)
-- **nicht überschrieben**, denn nur dann, wenn Personname unverändert bleibt
-- und gleichzeitig leer ist, wird eine neue Standardzusammensetzung erzeugt.

CREATE TRIGGER trg_person_update
BEFORE UPDATE ON Person
FOR EACH ROW
BEGIN
    -- Prüfen, ob die Spalte Personname im UPDATE geändert wird
    IF OLD.Personname = NEW.Personname THEN
        -- Die Spalte wird nicht verändert
        -- Optional: nur setzen, wenn sie leer ist
        IF NEW.Personname IS NULL OR NEW.Personname = '' THEN
            SET NEW.Personname = CONCAT(NEW.Personenvorname, ' ', NEW.Personennachname);
        END IF;
    END IF;
END;


-- Diese Prozedur ermöglicht das DSGVO-konforme Löschen eines Person-Eintrags,
-- ohne die referenzielle Integrität der Datenbank zu gefährden. Ein vollständiges
-- Entfernen der Datensätze ist nicht möglich, da dadurch Spieleabend-/Partie-
-- Daten, Teilnahmehistorien und Ranglisten inkonsistent würden oder ihre
-- Verknüpfungen verlieren würden.
-- Statt physischem Löschen werden daher alle personenbezogenen Daten
-- irreversibel anonymisiert und die Email durch eine eindeutige Dummy-Adresse
-- ersetzt. Auch verbundene Spieler-Daten (z. B. Nickname) werden neutralisiert.
-- Der Counter stellt sicher, dass jede Ersatzadresse eindeutig bleibt und die
-- technischen Strukturen (UNIQUE, Foreign Keys, Historien) weiterhin korrekt
-- funktionieren.

CREATE PROCEDURE anonymize_person(IN p_email VARCHAR(100))
BEGIN
    DECLARE current_counter INT;

    -- --------------------------------------------------------------------
    -- Counter laden:
    -- Dient der Erzeugung eindeutiger Dummy-Emailadressen, damit beim
    -- DSGVO-konformen Löschen technische Integrität (FK, UNIQUE) gewahrt
    -- bleibt, ohne dass echte personenbezogene Daten zurückbleiben.
    -- --------------------------------------------------------------------
    SELECT counter INTO current_counter FROM geloescht_counter WHERE id = 1;

    -- --------------------------------------------------------------------
    -- Person anonymisieren:
    -- Alle personenbezogenen Felder werden durch neutrale Standardwerte
    -- ersetzt. Die Email wird in eine nicht rückführbare, eindeutige
    -- Dummy-Adresse umgewandelt.
    -- So bleiben Verknüpfungen bestehen, ohne Personenbezug.
    -- --------------------------------------------------------------------
    UPDATE Person
    SET
        Personenvorname = 'Benutzer',
        Personennachname = 'gelöscht',
        Personname = 'Benutzer gelöscht',
        Email = CONCAT('geloescht', current_counter, '@example.com')
    WHERE Email = p_email;

    -- --------------------------------------------------------------------
    -- Nickname anonymisieren:
    -- Da der Spieler-Datensatz über die (nun pseudonymisierte) Email
    -- verknüpft ist und ein Nickname ebenfalls personenbezogen ist,
    -- wird er auf einen neutralen Wert ('-') gesetzt.
    -- Eindeutigkeit ist hier nicht erforderlich.
    -- --------------------------------------------------------------------
    UPDATE Spieler s
    JOIN Person p ON s.Email = p.Email
    SET s.Nickname = '-'
    WHERE p.Email = CONCAT('geloescht', current_counter, '@example.com');

    -- --------------------------------------------------------------------
    -- Counter erhöhen:
    -- Dadurch bleibt jede künftige Dummy-Adresse eindeutig und
    -- verhindert Datenkollisionen oder Verwechslungen.
    -- --------------------------------------------------------------------
    UPDATE geloescht_counter
    SET counter = counter + 1
    WHERE id = 1;
END;



CREATE OR REPLACE TRIGGER trg_Ausleihe_VorUpdate
BEFORE UPDATE ON Ausleihe
FOR EACH ROW
BEGIN
    DECLARE v_start DATE; -- Startdatum bestehender Ausleihe prüfen
    DECLARE v_end DATE; -- Enddatum bestehender Ausleihe prüfen
    DECLARE v_msg TEXT; -- Fehlermeldung für SIGNAL
    DECLARE v_partie INT; -- Partie_ID, falls Ausleiher nicht teilnimmt
    DECLARE v_besitzer_email VARCHAR(255); -- Besitzer der Kopie
    DECLARE v_is_root BOOLEAN DEFAULT FALSE; -- Root-Flag
    DECLARE v_is_dba BOOLEAN DEFAULT FALSE; -- DBA-Flag

    -- Besitzer der Kopie ermitteln
SELECT Besitzer_Email INTO v_besitzer_email
FROM Spielkopie
WHERE Kopie_ID = NEW.Kopie_ID
    LIMIT 1;

-- root erkennen
IF USER_EMAIL() = 'root' THEN SET v_is_root = TRUE; END IF;

    -- dba_role erkennen
    IF FIND_IN_SET('dba_role', CURRENT_ROLE()) THEN SET v_is_dba = TRUE; END IF;

    -- ----------------------------------------------------------------
    -- NEU: Logik-Check für Updates
    -- ----------------------------------------------------------------
    IF v_besitzer_email = NEW.Spieler_Email THEN
        SET v_msg = 'Ungültig: Der Besitzer kann nicht als Ausleiher für sein eigenes Spiel eingetragen werden.';
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
END IF;

    -- ----------------------------------------------------------------
    -- 0. Regel: Wer darf die Ausleihe ändern?
    -- ----------------------------------------------------------------
    IF USER_EMAIL() <> v_besitzer_email THEN
        IF v_is_root THEN
            SET @dummy = 0; -- Root darf alles
        ELSEIF v_is_dba THEN
            SET v_msg = CONCAT('Warnung (DBA): Ausleihe wird geändert, obwohl Benutzer nicht Besitzer ist. Kopie_ID=', NEW.Kopie_ID, ', Ausleihstartdatum=', NEW.Ausleihstartdatum, ', Besitzer=', v_besitzer_email);
            SIGNAL SQLSTATE '01000' SET MESSAGE_TEXT = v_msg; -- Warnung für DBA
ELSE
            SET v_msg = CONCAT('Ungültig: Nur der Besitzer (', v_besitzer_email, ') darf die Ausleihe ändern.');
            SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Fehler für unberechtigten Nutzer
END IF;
END IF;

    -- ----------------------------------------------------------------
    -- 1. Prüfen: Ausleihenddatum darf nicht kleiner als Startdatum sein
    -- ----------------------------------------------------------------
    IF NEW.Ausleihenddatum IS NOT NULL AND NEW.Ausleihenddatum < NEW.Ausleihstartdatum THEN
        SET v_msg = CONCAT('Ungültig: Ausleihenddatum (', NEW.Ausleihenddatum, ') liegt vor Ausleihstartdatum (', NEW.Ausleihstartdatum, ').');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
END IF;

    -- ----------------------------------------------------------------
    -- 2. Prüfen auf Überschneidungen mit bestehenden Ausleihen (außer dem eigenen Datensatz)
    -- ----------------------------------------------------------------
SELECT Ausleihstartdatum, Ausleihenddatum INTO v_start, v_end
FROM Ausleihe
WHERE Kopie_ID = NEW.Kopie_ID
  AND NOT (Kopie_ID = OLD.Kopie_ID AND Spieler_Email = OLD.Spieler_Email AND Ausleihstartdatum = OLD.Ausleihstartdatum)
  AND (
    (Ausleihenddatum IS NULL AND NEW.Ausleihstartdatum <= CURDATE())
        OR (NEW.Ausleihstartdatum <= IFNULL(Ausleihenddatum, NEW.Ausleihstartdatum)
        AND (NEW.Ausleihenddatum IS NULL OR NEW.Ausleihenddatum >= Ausleihstartdatum))
    )
    LIMIT 1;

IF v_start IS NOT NULL THEN
        SET v_msg = CONCAT('Kopie ist im gewünschten Zeitraum bereits ausgeliehen. Bestehende Ausleihe von ', v_start, ' bis ', IFNULL(v_end, 'noch offen'), '.');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
END IF;

    -- ----------------------------------------------------------------
    -- 3. Prüfen: Ausleiher ist Teilnehmer aller Partien im Zeitraum
    -- ----------------------------------------------------------------
SELECT p.Partie_ID INTO v_partie
FROM Partie p
         LEFT JOIN Teilnahme t ON p.Partie_ID = t.Partie_ID AND t.Spieler_Email = NEW.Spieler_Email
WHERE p.Spielkopie_ID = NEW.Kopie_ID
  AND (
    (NEW.Ausleihenddatum IS NULL AND p.Partiedatum >= NEW.Ausleihstartdatum)
        OR (NEW.Ausleihenddatum IS NOT NULL AND p.Partiedatum BETWEEN NEW.Ausleihstartdatum AND NEW.Ausleihenddatum)
    )
  AND t.Spieler_Email IS NULL
    LIMIT 1;

IF v_partie IS NOT NULL THEN
        SET v_msg = CONCAT('Die ausleihende Person ist nicht Teilnehmer der Partie ', v_partie, ' im gewünschten Ausleihzeitraum.');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
END IF;

END;



-- Trigger: trg_Ausleihe_VorUpdate
-- Zweck: Genauso wie trg_Ausleihe_VorInsert, jedoch für Updates.
-- Begründung:
-- Die Berechtigungsprüfung, Termin- und Überschneidungsprüfung sind identisch
-- zu BEFORE INSERT (siehe dort). Unterschied: Bei Updates muss der eigene Datensatz
-- bei der Überschneidungsprüfung ausgeschlossen werden, sonst würde jede Änderung blockiert.
-- So werden alle Datenkonsistenzen (Ausleihzeiträume, Partizipation, Berechtigungen)
-- weiterhin sichergestellt.

CREATE OR REPLACE TRIGGER trg_Ausleihe_VorUpdate
BEFORE UPDATE ON Ausleihe
FOR EACH ROW
BEGIN
    DECLARE v_start DATE; -- Startdatum bestehender Ausleihe prüfen
    DECLARE v_end DATE; -- Enddatum bestehender Ausleihe prüfen
    DECLARE v_msg TEXT; -- Fehlermeldung für SIGNAL
    DECLARE v_partie INT; -- Partie_ID, falls Ausleiher nicht teilnimmt
    DECLARE v_besitzer_email VARCHAR(255); -- Besitzer der Kopie
    DECLARE v_is_root BOOLEAN DEFAULT FALSE; -- Root-Flag
    DECLARE v_is_dba BOOLEAN DEFAULT FALSE; -- DBA-Flag

    -- Besitzer der Kopie ermitteln
    SELECT Besitzer_Email INTO v_besitzer_email
    FROM Spielkopie
    WHERE Kopie_ID = NEW.Kopie_ID
    LIMIT 1;

    -- root erkennen
    IF USER_EMAIL() = 'root' THEN SET v_is_root = TRUE; END IF;

    -- dba_role erkennen
    IF FIND_IN_SET('dba_role', CURRENT_ROLE()) THEN SET v_is_dba = TRUE; END IF;

    -- ----------------------------------------------------------------
    -- 0. Regel: Wer darf die Ausleihe ändern?
    -- ----------------------------------------------------------------
    IF USER_EMAIL() <> v_besitzer_email THEN
        IF v_is_root THEN
            SET @dummy = 0; -- Root darf alles
        ELSEIF v_is_dba THEN
            SET v_msg = CONCAT('Warnung (DBA): Ausleihe wird geändert, obwohl Benutzer nicht Besitzer ist. Kopie_ID=', NEW.Kopie_ID, ', Ausleihstartdatum=', NEW.Ausleihstartdatum, ', Besitzer=', v_besitzer_email);
            SIGNAL SQLSTATE '01000' SET MESSAGE_TEXT = v_msg; -- Warnung für DBA
        ELSE
            SET v_msg = CONCAT('Ungültig: Nur der Besitzer (', v_besitzer_email, ') darf die Ausleihe ändern.');
            SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Fehler für unberechtigten Nutzer
        END IF;
    END IF;

    -- ----------------------------------------------------------------
    -- 1. Prüfen: Ausleihenddatum darf nicht kleiner als Startdatum sein
    -- ----------------------------------------------------------------
    IF NEW.Ausleihenddatum IS NOT NULL AND NEW.Ausleihenddatum < NEW.Ausleihstartdatum THEN
        SET v_msg = CONCAT('Ungültig: Ausleihenddatum (', NEW.Ausleihenddatum, ') liegt vor Ausleihstartdatum (', NEW.Ausleihstartdatum, ').');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
    END IF;

    -- ----------------------------------------------------------------
    -- 2. Prüfen auf Überschneidungen mit bestehenden Ausleihen (außer dem eigenen Datensatz)
    -- ----------------------------------------------------------------
    SELECT Ausleihstartdatum, Ausleihenddatum INTO v_start, v_end
    FROM Ausleihe
    WHERE Kopie_ID = NEW.Kopie_ID
      AND NOT (Kopie_ID = OLD.Kopie_ID AND Spieler_Email = OLD.Spieler_Email)
      AND (
            (Ausleihenddatum IS NULL AND NEW.Ausleihstartdatum <= CURDATE())
            OR (NEW.Ausleihstartdatum <= IFNULL(Ausleihenddatum, NEW.Ausleihstartdatum)
                AND (NEW.Ausleihenddatum IS NULL OR NEW.Ausleihenddatum >= Ausleihstartdatum))
          )
    LIMIT 1;

    IF v_start IS NOT NULL THEN
        SET v_msg = CONCAT('Kopie ist im gewünschten Zeitraum bereits ausgeliehen. Bestehende Ausleihe von ', v_start, ' bis ', IFNULL(v_end, 'noch offen'), '.');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
    END IF;

    -- ----------------------------------------------------------------
    -- 3. Prüfen: Ausleiher ist Teilnehmer aller Partien im Zeitraum
    -- ----------------------------------------------------------------
    SELECT p.Partie_ID INTO v_partie
    FROM Partie p
    LEFT JOIN Teilnahme t ON p.Partie_ID = t.Partie_ID AND t.Spieler_Email = NEW.Spieler_Email
    WHERE p.Spielkopie_ID = NEW.Kopie_ID
      AND (
            (NEW.Ausleihenddatum IS NULL AND p.Partiedatum >= NEW.Ausleihstartdatum)
            OR (NEW.Ausleihenddatum IS NOT NULL AND p.Partiedatum BETWEEN NEW.Ausleihstartdatum AND NEW.Ausleihenddatum)
          )
      AND t.Spieler_Email IS NULL
    LIMIT 1;

    IF v_partie IS NOT NULL THEN
        SET v_msg = CONCAT('Die ausleihende Person ist nicht Teilnehmer der Partie ', v_partie, ' im gewünschten Ausleihzeitraum.');
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
    END IF;

END;


-- Trigger: trg_Spieleranzahl_Check_Insert
-- Zweck: Sicherstellen, dass für ein Spiel die Minimal- und Maximalspielerzahl
--        logisch konsistent sind (Spieleranzahl_min <= Spieleranzahl_max).
-- Begründung:
-- 1. Datenkonsistenz: Ohne diese Prüfung könnten fehlerhafte Werte
--    eingetragen werden, die spätere Validierungen oder Spielauswertungen
--    unmöglich machen.
-- 2. Funktionsweise: Der Trigger prüft vor dem Einfügen die beiden Werte
--    und löst bei Inkonsistenz einen SQL-Fehler aus, sodass das Einfügen
--    abgelehnt wird.

CREATE OR REPLACE TRIGGER trg_Spieleranzahl_Check_Insert
BEFORE INSERT ON Spiel_Spieleranzahl
FOR EACH ROW
BEGIN
    DECLARE v_msg TEXT; -- Fehlermeldungstext

    IF NEW.Spieleranzahl_min > NEW.Spieleranzahl_max THEN
        SET v_msg = CONCAT(
            'Ungültig: Spieleranzahl_min (', NEW.Spieleranzahl_min,
            ') darf nicht größer sein als Spieleranzahl_max (', NEW.Spieleranzahl_max, ').'
        );
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Abbruch bei Inkonsistenz
    END IF;
END;

-- Trigger: trg_Spieleranzahl_Check_Update
-- Zweck: Genauso wie trg_Spieleranzahl_Check_Insert, jedoch für Updates.
-- Begründung:
-- Die gleiche Logik muss auch bei Änderungen überprüft werden, damit die
-- Konsistenz von Spieleranzahl_min und -max erhalten bleibt.  
-- Der Trigger verweist inhaltlich auf die Prüfungen des Insert-Triggers.

CREATE OR REPLACE TRIGGER trg_Spieleranzahl_Check_Update
BEFORE UPDATE ON Spiel_Spieleranzahl
FOR EACH ROW
BEGIN
    DECLARE v_msg TEXT;

    IF NEW.Spieleranzahl_min > NEW.Spieleranzahl_max THEN
        SET v_msg = CONCAT(
            'Ungültig: Spieleranzahl_min (', NEW.Spieleranzahl_min,
            ') darf nicht größer sein als Spieleranzahl_max (', NEW.Spieleranzahl_max, ').'
        );
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
    END IF;
END;


-- Trigger: trg_Spieldauer_Check_Insert
-- Zweck: Sicherstellen, dass die minimale Spieldauer eines Spiels nicht größer
--        ist als die maximale Spieldauer.
-- Begründung:
-- 1. Datenkonsistenz: Ohne diese Prüfung könnten inkonsistente Werte gespeichert
--    werden, die spätere Berechnungen oder Spielauswertungen unmöglich machen.
-- 2. Funktionsweise: Prüft vor dem Einfügen die Werte und bricht die Operation
--    mit einem SQL-Fehler ab, wenn min > max.

CREATE OR REPLACE TRIGGER trg_Spieldauer_Check_Insert
BEFORE INSERT ON Spiel_Spieldauer
FOR EACH ROW
BEGIN
    DECLARE v_msg TEXT; -- Fehlermeldungstext

    IF NEW.Spieldauer_min > NEW.Spieldauer_max THEN
        SET v_msg = CONCAT(
            'Ungültig: Spieldauer_min (', NEW.Spieldauer_min,
            ') darf nicht größer sein als Spieldauer_max (', NEW.Spieldauer_max, ').'
        );
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Abbruch bei Inkonsistenz
    END IF;
END;

-- Trigger: trg_Spieldauer_Check_Update
-- Zweck: Analog zum Insert-Trigger, aber bei Updates.
-- Begründung:
-- Die gleiche Prüfung muss auch bei Änderungen erfolgen, um die Konsistenz
-- von Spieldauer_min und Spieldauer_max zu erhalten. Funktional entspricht
-- dieser Trigger dem Insert-Trigger.

CREATE OR REPLACE TRIGGER trg_Spieldauer_Check_Update
BEFORE UPDATE ON Spiel_Spieldauer
FOR EACH ROW
BEGIN
    DECLARE v_msg TEXT;

    IF NEW.Spieldauer_min > NEW.Spieldauer_max THEN
        SET v_msg = CONCAT(
            'Ungültig: Spieldauer_min (', NEW.Spieldauer_min,
            ') darf nicht größer sein als Spieldauer_max (', NEW.Spieldauer_max, ').'
        );
        SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg;
    END IF;
END;




-- Trigger: trg_Partie_Validate_Insert
-- --------------------------------------------------------------------
-- Zweck:
-- Dieser Trigger stellt sicher, dass beim Einfügen eines Datensatzes in
-- die Tabelle `Partie` sämtliche Integritäts- und Berechtigungsregeln
-- eingehalten werden.  
-- 
-- Validierungsziele:
-- 1. Das Partiedatum muss mit dem zugehörigen Spieleabenddatum übereinstimmen.
-- 2. Die Berechtigung des Benutzers zum Anlegen der Partie wird überprüft,
--    abhängig davon, ob die Spielkopie ausgeliehen ist oder nicht.
-- 3. Root- und DBA-Rollen erhalten Ausnahmen, normale Benutzer werden strikt
--    geprüft.  
-- 
-- Motivation:
-- Sicherstellung von Datenkonsistenz, korrekten Zugriffsrechten und
-- nachvollziehbaren Fehlermeldungen für die Nutzer.
-- --------------------------------------------------------------------


CREATE OR REPLACE TRIGGER trg_Partie_Validate_Insert
BEFORE INSERT ON Partie
FOR EACH ROW
BEGIN
    DECLARE v_spieleabenddatum DATE;        -- Datum des zugehörigen Spieleabends
    DECLARE v_besitzer_email VARCHAR(255);  -- Besitzer der Spielkopie
    DECLARE v_ausleiher_email VARCHAR(255); -- Aktueller Ausleiher der Kopie, falls vorhanden
    DECLARE v_msg TEXT;                      -- Fehlermeldungstext
    DECLARE v_is_root BOOLEAN DEFAULT FALSE;-- Flag, ob Benutzer Root ist
    DECLARE v_is_dba BOOLEAN DEFAULT FALSE; -- Flag, ob Benutzer DBA-Rolle innehat


    -- ----------------------------------------------------------------
    -- 1. Validierung: Übereinstimmung Partiedatum mit Spieleabenddatum
    -- ----------------------------------------------------------------
    IF NEW.Spieleabend_ID IS NOT NULL THEN
        SELECT Spieleabenddatum
        INTO v_spieleabenddatum
        FROM Spieleabend
        WHERE Spieleabend_ID = NEW.Spieleabend_ID
        LIMIT 1; -- Abfrage des korrespondierenden Spieleabenddatums

        IF NEW.Partiedatum <> v_spieleabenddatum THEN
            SET v_msg = CONCAT(
                'Ungültig: Partiedatum (', NEW.Partiedatum,
                ') muss mit Spieleabenddatum (', v_spieleabenddatum, ') übereinstimmen.'
            );
            SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Abbruch bei Verletzung der Regel
        END IF;
    END IF;


    -- ----------------------------------------------------------------
    -- 2. Ermittlung des Besitzers der Spielkopie
    -- ----------------------------------------------------------------
    SELECT Besitzer_Email
    INTO v_besitzer_email
    FROM Spielkopie
    WHERE Kopie_ID = NEW.Spielkopie_ID
    LIMIT 1; -- Zur Berechtigungsprüfung des Benutzers


    -- ----------------------------------------------------------------
    -- 3. Prüfen, ob die Spielkopie am Partiedatum ausgeliehen ist
    -- ----------------------------------------------------------------
    SELECT Spieler_Email
    INTO v_ausleiher_email
    FROM Ausleihe
    WHERE Kopie_ID = NEW.Spielkopie_ID
      AND NEW.Partiedatum >= Ausleihstartdatum
      AND (Ausleihenddatum IS NULL OR NEW.Partiedatum <= Ausleihenddatum)
    LIMIT 1; -- Ermittlung des aktuellen Ausleihers, falls vorhanden


    -- ----------------------------------------------------------------
    -- 4. Berechtigungsprüfung: Wer darf die Partie anlegen?
    --    Unterscheidung zwischen ausgeliehener und nicht ausgeliehener Kopie
    -- ----------------------------------------------------------------
    IF USER_EMAIL() = 'root' THEN
        SET v_is_root = TRUE; -- Root-Benutzer erhält uneingeschränkten Zugriff
    END IF;

    IF FIND_IN_SET('dba_role', CURRENT_ROLE()) THEN
        SET v_is_dba = TRUE; -- DBA-Benutzer erhält eingeschränkte Ausnahmeregelung
    END IF;


    -- Fall A: Kopie ist ausgeliehen
    IF v_ausleiher_email IS NOT NULL THEN
        IF v_ausleiher_email <> USER_EMAIL() THEN
            IF v_is_root THEN
                SET @dummy = 0; -- Root darf beliebige Aktionen ausführen
            ELSEIF v_is_dba THEN
                SET v_msg = CONCAT(
                    'Warnung (DBA): Partie angelegt trotz ausgeliehener Kopie. ',
                    'Kopie_ID=', NEW.Spielkopie_ID,
                    ', Partiedatum=', NEW.Partiedatum,
                    ', Ausleiher=', v_ausleiher_email
                );
                SIGNAL SQLSTATE '01000' SET MESSAGE_TEXT = v_msg; -- Warnung, keine Blockade
            ELSE
                SET v_msg = CONCAT(
                    'Ungültig: Die Kopie ist am ', NEW.Partiedatum,
                    ' an ', v_ausleiher_email, ' ausgeliehen. ',
                    'Nur der Ausleiher darf an diesem Datum eine Partie anlegen.'
                );
                SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Harte Fehlerausgabe
            END IF;
        END IF;

    -- Fall B: Kopie ist nicht ausgeliehen
    ELSE
        IF v_besitzer_email <> USER_EMAIL() THEN
            IF v_is_root THEN
                SET @dummy = 0; -- Root darf beliebige Aktionen ausführen
            ELSEIF v_is_dba THEN
                SET v_msg = CONCAT(
                    'Warnung (DBA): Partie angelegt ohne Besitzer zu sein. ',
                    'Kopie_ID=', NEW.Spielkopie_ID,
                    ', Partiedatum=', NEW.Partiedatum,
                    ', Besitzer=', v_besitzer_email
                );
                SIGNAL SQLSTATE '01000' SET MESSAGE_TEXT = v_msg; -- Warnung, keine Blockade
            ELSE
                SET v_msg = CONCAT(
                    'Ungültig: Die Kopie ist nicht ausgeliehen. ',
                    'Nur der Besitzer (', v_besitzer_email,
                    ') darf eine Partie mit dieser Kopie anlegen.'
                );
                SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Harte Fehlerausgabe
            END IF;
        END IF;
    END IF;

END;


-- Trigger: trg_Partie_Validate_Update
-- --------------------------------------------------------------------
-- Zweck:
-- Dieser Trigger stellt sicher, dass beim Aktualisieren eines Datensatzes 
-- in der Tabelle `Partie` sämtliche Integritäts- und Berechtigungsregeln
-- eingehalten werden.  
-- Begründung:
-- 1. Aktualisierungen können Partiedatum, Spielkopie oder Spieler_Email betreffen.
-- 2. Ohne diese Prüfung könnten Inkonsistenzen entstehen, z.B.:
--    - Partien an falschen Terminen
--    - Ausführung durch unberechtigte Benutzer
--    - Konflikte mit Ausleihen oder Spieleabenddaten
-- 3. Root- und DBA-Ausnahmen gelten analog zum INSERT-Trigger.
-- Hinweis: Die konkrete Logik entspricht trg_Partie_Validate_Insert.  
-- --------------------------------------------------------------------


CREATE OR REPLACE TRIGGER trg_Partie_Validate_Update
BEFORE UPDATE ON Partie
FOR EACH ROW
BEGIN
    DECLARE v_spieleabenddatum DATE;        -- Datum des zugehörigen Spieleabends
    DECLARE v_besitzer_email VARCHAR(255);  -- Besitzer der Spielkopie
    DECLARE v_ausleiher_email VARCHAR(255); -- Aktueller Ausleiher der Kopie, falls vorhanden
    DECLARE v_msg TEXT;                      -- Fehlermeldungstext
    DECLARE v_is_root BOOLEAN DEFAULT FALSE;-- Flag, ob Benutzer Root ist
    DECLARE v_is_dba BOOLEAN DEFAULT FALSE; -- Flag, ob Benutzer DBA-Rolle innehat


    -- ----------------------------------------------------------------
    -- 1. Validierung: Übereinstimmung Partiedatum mit Spieleabenddatum
    -- ----------------------------------------------------------------
    IF NEW.Spieleabend_ID IS NOT NULL THEN
        SELECT Spieleabenddatum
        INTO v_spieleabenddatum
        FROM Spieleabend
        WHERE Spieleabend_ID = NEW.Spieleabend_ID
        LIMIT 1; -- Abfrage des korrespondierenden Spieleabenddatums

        IF NEW.Partiedatum <> v_spieleabenddatum THEN
            SET v_msg = CONCAT(
                'Ungültig: Partiedatum (', NEW.Partiedatum,
                ') muss mit Spieleabenddatum (', v_spieleabenddatum, ') übereinstimmen.'
            );
            SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Abbruch bei Verletzung der Regel
        END IF;
    END IF;


    -- ----------------------------------------------------------------
    -- 2. Ermittlung des Besitzers der Spielkopie
    -- ----------------------------------------------------------------
    SELECT Besitzer_Email
    INTO v_besitzer_email
    FROM Spielkopie
    WHERE Kopie_ID = NEW.Spielkopie_ID
    LIMIT 1; -- Zur Berechtigungsprüfung des Benutzers


    -- ----------------------------------------------------------------
    -- 3. Prüfen, ob die Spielkopie am Partiedatum ausgeliehen ist
    -- ----------------------------------------------------------------
    SELECT Spieler_Email
    INTO v_ausleiher_email
    FROM Ausleihe
    WHERE Kopie_ID = NEW.Spielkopie_ID
      AND NEW.Partiedatum >= Ausleihstartdatum
      AND (Ausleihenddatum IS NULL OR NEW.Partiedatum <= Ausleihenddatum)
    LIMIT 1; -- Ermittlung des aktuellen Ausleihers, falls vorhanden


    -- ----------------------------------------------------------------
    -- 4. Berechtigungsprüfung: Wer darf die Partie anlegen?
    --    Unterscheidung zwischen ausgeliehener und nicht ausgeliehener Kopie
    -- ----------------------------------------------------------------
    IF USER_EMAIL() = 'root' THEN
        SET v_is_root = TRUE; -- Root-Benutzer erhält uneingeschränkten Zugriff
    END IF;

    IF FIND_IN_SET('dba_role', CURRENT_ROLE()) THEN
        SET v_is_dba = TRUE; -- DBA-Benutzer erhält eingeschränkte Ausnahmeregelung
    END IF;


    -- Fall A: Kopie ist ausgeliehen
    IF v_ausleiher_email IS NOT NULL THEN
        IF v_ausleiher_email <> USER_EMAIL() THEN
            IF v_is_root THEN
                SET @dummy = 0; -- Root darf beliebige Aktionen ausführen
            ELSEIF v_is_dba THEN
                SET v_msg = CONCAT(
                    'Warnung (DBA): Partie angelegt trotz ausgeliehener Kopie. ',
                    'Kopie_ID=', NEW.Spielkopie_ID,
                    ', Partiedatum=', NEW.Partiedatum,
                    ', Ausleiher=', v_ausleiher_email
                );
                SIGNAL SQLSTATE '01000' SET MESSAGE_TEXT = v_msg; -- Warnung, keine Blockade
            ELSE
                SET v_msg = CONCAT(
                    'Ungültig: Die Kopie ist am ', NEW.Partiedatum,
                    ' an ', v_ausleiher_email, ' ausgeliehen. ',
                    'Nur der Ausleiher darf an diesem Datum eine Partie anlegen.'
                );
                SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Harte Fehlerausgabe
            END IF;
        END IF;

    -- Fall B: Kopie ist nicht ausgeliehen
    ELSE
        IF v_besitzer_email <> USER_EMAIL() THEN
            IF v_is_root THEN
                SET @dummy = 0; -- Root darf beliebige Aktionen ausführen
            ELSEIF v_is_dba THEN
                SET v_msg = CONCAT(
                    'Warnung (DBA): Partie angelegt ohne Besitzer zu sein. ',
                    'Kopie_ID=', NEW.Spielkopie_ID,
                    ', Partiedatum=', NEW.Partiedatum,
                    ', Besitzer=', v_besitzer_email
                );
                SIGNAL SQLSTATE '01000' SET MESSAGE_TEXT = v_msg; -- Warnung, keine Blockade
            ELSE
                SET v_msg = CONCAT(
                    'Ungültig: Die Kopie ist nicht ausgeliehen. ',
                    'Nur der Besitzer (', v_besitzer_email,
                    ') darf eine Partie mit dieser Kopie anlegen.'
                );
                SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = v_msg; -- Harte Fehlerausgabe
            END IF;
        END IF;
    END IF;

END;



