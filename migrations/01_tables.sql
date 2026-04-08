-- --------------------------------------------------------
-- Tabelle: VERLAG_ADRESSE (mehrwertiges Attribut)
-- --------------------------------------------------------
CREATE TABLE Verlag_Adresse (
  Adresse_ID INT(11) NOT NULL AUTO_INCREMENT,
  -- Erlaubt eindeutige Referenz auf die Adresse
  PLZ VARCHAR(10) COLLATE utf8_unicode_ci NOT NULL,
  -- Essentieller Teil der Adresse, um Post zu versenden
  Ort VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Ebenfalls essentieller Teil der Anschrift
  Strasse VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Teil der Adresse
  Hausnummer VARCHAR(5) COLLATE utf8_unicode_ci NOT NULL,
  -- Teil der Adresse (Länge ist mit Puffer für unvorhergesehene Addressen)
  PRIMARY KEY(Adresse_ID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
-- Gleiche Kodierung für Konsistenz und Sprachunabhängigkeit

-- --------------------------------------------------------
-- Tabelle: VERLAG
-- --------------------------------------------------------
CREATE TABLE Verlag (
  Verlagsname VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Speichert den Herausgeber eines Spiels, um Veröffentlichungen bestimmten Unternehmen zuordnen zu können
  Verlagsadresse INT(11) NOT NULL,
  -- Referenz auf Adresse für regionale Auswertungen und zur Kontaktaufnahme bei Problemen
  PRIMARY KEY (Verlagsname),
  -- Der Name eines Herausgebers wird als eindeutiger Schlüssel genutzt, da er in der Branche identifizierend wirkt
  FOREIGN KEY (Verlagsadresse) REFERENCES Verlag_Adresse(Adresse_ID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
-- Einheitlicher Zeichensatz für internationale Verlagsnamen

-- --------------------------------------------------------
-- Tabelle: SPIEL
-- --------------------------------------------------------
CREATE TABLE Spiel (
  Spieltitel VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  Erscheinungsjahr YEAR(4) NOT NULL,
  Kategorie VARCHAR(50) COLLATE utf8_unicode_ci DEFAULT NULL,
  Altersempfehlung INT DEFAULT NULL,
  Verlagsname VARCHAR(100) COLLATE utf8_unicode_ci DEFAULT NULL,
  Spieldauer_Durchschnitt INTEGER DEFAULT NULL,
  BGG_Rating FLOAT DEFAULT 0.0, -- Diese Zeile muss vorhanden sein
  PRIMARY KEY (Spieltitel, Erscheinungsjahr),
  FOREIGN KEY (Verlagsname) REFERENCES Verlag(Verlagsname)
    ON DELETE SET NULL
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

-- --------------------------------------------------------
-- Tabelle: SPIEL_SPIELERANZAHL
-- --------------------------------------------------------
CREATE TABLE Spiel_Spieleranzahl (
  Spieltitel VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  Erscheinungsjahr YEAR(4) NOT NULL,
  Spieleranzahl_min INT NOT NULL,
  Spieleranzahl_max INT NOT NULL,
  FOREIGN KEY (Spieltitel, Erscheinungsjahr) REFERENCES Spiel(Spieltitel, Erscheinungsjahr)
    ON DELETE CASCADE
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
-- Zahlenfelder gewählt, um Auswertungen (z. B. Durchschnittswerte, Filter) effizient zu unterstützen

-- --------------------------------------------------------
-- Tabelle: SPIEL_SPIELDAUER
-- --------------------------------------------------------
CREATE TABLE Spiel_Spieldauer (
  Spieltitel VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  Erscheinungsjahr YEAR(4) NOT NULL,
  Spieldauer_min INT NOT NULL,
  Spieldauer_max INT NOT NULL,
  FOREIGN KEY (Spieltitel, Erscheinungsjahr) REFERENCES Spiel(Spieltitel, Erscheinungsjahr)
    ON DELETE CASCADE
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
-- Zahlenform erlaubt spätere Filterung nach Dauerintervallen in Abfragen
-- Einheitliches Encoding für Textfelder (Titel, Jahr)

-- --------------------------------------------------------
-- Tabelle: PERSON
-- --------------------------------------------------------
CREATE TABLE Person (
  Email VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  Personname VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  Personenvorname VARCHAR(50) COLLATE utf8_unicode_ci NOT NULL,
  Personennachname VARCHAR(50) COLLATE utf8_unicode_ci NOT NULL,
  Notizen TEXT COLLATE utf8_unicode_ci DEFAULT NULL,
  PRIMARY KEY (Email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

CREATE INDEX idx_person_personname
ON Person (Personname);

CREATE INDEX idx_person_personenvorname
ON Person (Personenvorname);

CREATE INDEX idx_person_personennachname
ON Person (Personennachname);

CREATE INDEX idx_person_kombiniert
ON Person (Personenvorname, Personennachname);

-- --------------------------------------------------------
-- Tabelle: AUTOR (Untertyp von Person)
-- --------------------------------------------------------
CREATE TABLE Autor (
  Autor_Email VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Repräsentiert eine Person, die ein Spiel entworfen hat
  PRIMARY KEY (Autor_Email),
  FOREIGN KEY (Autor_Email) REFERENCES Person(Email)
    ON DELETE CASCADE
  -- Entfernt den Autoreneintrag automatisch, wenn die zugrunde liegende Person gelöscht wird
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
-- Einheitliche Zeichensatzwahl zur Vermeidung von Vergleichsfehlern zwischen Basistabelle und Spezialisierung

-- --------------------------------------------------------
-- Tabelle: SPIELER (Untertyp von Person)
-- --------------------------------------------------------
CREATE TABLE Spieler (
  Email VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Verbindung zur Basistabelle Person, da Spieler reale Mitglieder oder Freunde sind
  Nickname VARCHAR(50) COLLATE utf8_unicode_ci UNIQUE,
  -- Pseudonym zur Anzeige in Statistiken oder für Ranglisten
  -- Macht das System attraktiver für neue Nutzer (Customizing)
  PRIMARY KEY (Email),
  FOREIGN KEY (Email) REFERENCES Person(Email)
    ON DELETE CASCADE
  -- Löscht automatisch die Spielerdaten, wenn die übergeordnete Person entfernt wird
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
-- UTF-8, um auch kreative Nicknamen (Sonderzeichen, Umlaute) zuzulassen

-- --------------------------------------------------------
-- Tabelle: SPIELKOPIE (schwacher Entitätstyp)
-- --------------------------------------------------------
CREATE TABLE Spielkopie (
  Kopie_ID INT(11) NOT NULL AUTO_INCREMENT,
  -- Identifiziert ein physisches Exemplar eindeutig
  Spieltitel VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Verbindung zum Spiel, von dem diese Kopie stammt
  Erscheinungsjahr YEAR(4) NOT NULL,
  -- Verbindung zum Spiel, von dem diese Kopie stammt
  Besitzer_Email VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Ermöglicht Nachverfolgung, wem eine Kopie gehört
  PRIMARY KEY (Kopie_ID),
  FOREIGN KEY (Spieltitel, Erscheinungsjahr) REFERENCES Spiel(Spieltitel, Erscheinungsjahr)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
  FOREIGN KEY (Besitzer_Email) REFERENCES Spieler(Email)
    ON DELETE CASCADE
  -- Löscht Kopien automatisch, wenn der Besitzer nicht mehr existiert oder das Spiel entfällt
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci AUTO_INCREMENT=1;

-- --------------------------------------------------------
-- Tabelle: DESIGNS (N:M Beziehung Spiel – Autor)
-- --------------------------------------------------------
CREATE TABLE Designs (
  Spieltitel VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Verbindung zum entworfenen Spiel
  Erscheinungsjahr YEAR(4) NOT NULL,
  -- Verbindung zum entworfenen Spiel
  Autor_Email VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Referenz auf den Gestalter, um seine Mitwirkungshistorie festzuhalten
  PRIMARY KEY (Spieltitel, Erscheinungsjahr, Autor_Email),
  FOREIGN KEY (Spieltitel, Erscheinungsjahr) REFERENCES Spiel(Spieltitel, Erscheinungsjahr)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
  FOREIGN KEY (Autor_Email) REFERENCES Autor(Autor_Email)
    ON DELETE CASCADE
  -- Verknüpfung entfällt, wenn entweder das Spiel oder der Autor gelöscht wird
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
-- Einheitliche Zeichensatzwahl für saubere Verknüpfungen zwischen Spiel- und Personendaten

-- --------------------------------------------------------
-- Tabelle: AUSLEIHE (N:M Beziehung Spielkopie – Spieler)
-- --------------------------------------------------------
CREATE TABLE Ausleihe (
  Kopie_ID INT(11) NOT NULL,
  -- Referenz auf das entliehene Exemplar
  Spieler_Email VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL DEFAULT 'geloescht@example.com',
  -- Identifiziert, wer eine Kopie aktuell oder früher ausgeliehen hat
  Ausleihstartdatum DATE NOT NULL,
  -- Erfasst den Beginn des Leihzeitraums, um Historien und Rückgabefristen nachzuhalten
  Ausleihenddatum DATE DEFAULT NULL,
  -- Optionales Enddatum zur Unterscheidung aktiver und abgeschlossener Ausleihen
  PRIMARY KEY (Kopie_ID, Spieler_Email, Ausleihstartdatum),
  FOREIGN KEY (Kopie_ID) REFERENCES Spielkopie(Kopie_ID)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
  FOREIGN KEY (Spieler_Email) REFERENCES Spieler(Email)
    ON DELETE SET DEFAULT
  -- Entfernt Leihdatensätze automatisch bei Löschung von Spieler oder Kopie
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
-- Einheitliches Encoding für E-Mails und internationale Namen

-- --------------------------------------------------------
-- Tabelle: SPIELEABEND
-- --------------------------------------------------------
CREATE TABLE Spieleabend (
  Spieleabend_ID INT(11) NOT NULL AUTO_INCREMENT,
  -- Dient der eindeutigen Identifikation eines Treffens
  Spieleabendnotizen TEXT COLLATE utf8_unicode_ci,
  -- Speichert Eindrücke, Ergebnisse oder organisatorische Hinweise
  Gastgeber_Email VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL DEFAULT 'geloescht@example.com',
  -- Verknüpft den Abend mit der Person, die ihn organisiert hat
  -- DEFAULT-Wert zum datenschutzfreundlichen Löschen einer Person
  Spieleabenddatum DATE NOT NULL,
  PRIMARY KEY (Spieleabend_ID),
  FOREIGN KEY (Gastgeber_Email) REFERENCES Person(Email)
    ON DELETE SET DEFAULT 
  -- Falls der Organisator entfällt, bleibt der Abend als Ereignis erhalten
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci AUTO_INCREMENT=1;
-- UTF-8 erlaubt die Speicherung von Freitexten mit Umlauten und Symbolen

-- --------------------------------------------------------
-- Tabelle: PARTIE
-- --------------------------------------------------------
CREATE TABLE Partie (
  Partie_ID INT(11) NOT NULL AUTO_INCREMENT,
  -- Eindeutige Kennung für jede gespielte Runde
  Partiedatum DATE NOT NULL,
  -- Ermöglicht zeitliche Auswertungen über Spielaktivität
  Spielkopie_ID INT(11) NOT NULL,
  -- Verbindung zur tatsächlich genutzten physischen Kopie
  Spieleabend_ID INT(11) DEFAULT NULL,
  -- Optionale Zuordnung zu einem Spieleabend, falls die Partie Teil eines Treffens war
  PRIMARY KEY (Partie_ID),
  FOREIGN KEY (Spielkopie_ID) REFERENCES Spielkopie(Kopie_ID)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
  FOREIGN KEY (Spieleabend_ID) REFERENCES Spieleabend(Spieleabend_ID)
    ON DELETE SET NULL
  -- Behält gespielte Partien für Statistiken auch dann bei, wenn der zugehörige Abend entfällt
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci AUTO_INCREMENT=1;
-- Einheitliches Encoding für Text- und Zeitdaten

-- --------------------------------------------------------
-- Tabelle: TEILNAHME (N:M Beziehung Spieler – Partie)
-- --------------------------------------------------------
CREATE TABLE Teilnahme (
  Partie_ID INT(11) NOT NULL,
  -- Identifiziert die gespielte Runde
  Spieler_Email VARCHAR(100) COLLATE utf8_unicode_ci NOT NULL,
  -- Verknüpft die Runde mit allen Mitspielenden zur späteren Auswertung
  Spielerrang INT(11),
  -- Optionaler Rang zur Unterscheidung geplanter und ausgetragener Partien
  PRIMARY KEY (Partie_ID, Spieler_Email),
  FOREIGN KEY (Partie_ID) REFERENCES Partie(Partie_ID)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
  FOREIGN KEY (Spieler_Email) REFERENCES Spieler(Email)
    ON DELETE CASCADE 
  -- Entfernt Teilnahmen automatisch, wenn Partien oder Spieler gelöscht werden
    ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;
-- Einheitlicher Zeichensatz, da Spielernamen und Mails internationale Zeichen enthalten können




