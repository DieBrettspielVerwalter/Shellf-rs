{ inputs, ... }:
{
  perSystem = { pkgs, system, src, self', ... }:
  let
    # Das mantra-Paket aus den Flake-Inputs
    mantra = inputs.mantra.packages.${system}.default;
    
    # Pfade zu den statischen Dateien (relativ zu diesem Modul)
    mantraToolsDir = ../tools/mantra;
    requirementsJson = "${mantraToolsDir}/requirements.json";
    reviewsDir = "${mantraToolsDir}/reviews";
    
    # Die final-coverage.json (angenommen sie wurde im anderen Modul definiert)
    # Falls sie noch nicht in perSystem verfügbar ist, hier Platzhalter:
    finalCoverageJson = self'.packages.final-coverage;

    # 1. Dynamische mantra.toml erzeugen
    mantraConfig = pkgs.writeText "mantra.toml" ''
      [project]
      name = "Shellf"
      version = "1.0.0"

      [[requirements]]
      files = ["${requirementsJson}"]

      [[traces]]
      root = "${src}"

      [coverage]
      # Pfad zur finalen Coverage-Datei aus dem vorherigen Schritt
      files = ["${self'.packages.final-coverage.outPath}/coverage.json"] 

      [review]
      files = [
        "${reviewsDir}/nfr.001.toml",
        "${reviewsDir}/nfr.002.toml",
        "${reviewsDir}/nfr.003.toml",
        "${reviewsDir}/nfr.004.toml",
        "${reviewsDir}/nfr.005.toml",
        "${reviewsDir}/nfr.006.toml",
        "${reviewsDir}/nfr.007.toml",
        "${reviewsDir}/nfr.008.toml",
        "${reviewsDir}/nfr.009.toml",
        "${reviewsDir}/nfr.010.toml",
        "${reviewsDir}/nfr.012.toml",
        "${reviewsDir}/nfr.013.toml",
        "${reviewsDir}/nfr.014.toml"
      ]
    '';

    # 2. Derivation für die mantra.db (Collect)
    mantra-db = pkgs.stdenv.mkDerivation {
      name = "mantra-database";
      nativeBuildInputs = [ mantra ];
      
      # Wir brauchen keinen echten Source-Tree zum Bauen, 
      # da mantra collect nur liest und eine DB schreibt
      phases = [ "buildPhase" "installPhase" ];

      buildPhase = ''
        echo "Collecting information into mantra.db..."
        # Mantra braucht Schreibrechte im aktuellen Verzeichnis für die SQLite DB
        mantra collect ${mantraConfig}
      '';

      installPhase = ''
        mkdir -p $out
        cp mantra.db $out/
      '';
    };

    # 3. Derivation für den HTML-Report
    mantra-report = pkgs.stdenv.mkDerivation {
      name = "mantra-report-html";
      nativeBuildInputs = [ mantra ];
      
      phases = [ "buildPhase" "installPhase" ];

      buildPhase = ''
        # Wir kopieren die DB aus der vorherigen Derivation, 
        # da mantra report sie im Arbeitsverzeichnis erwartet
        cp ${mantra-db}/mantra.db .
        
        echo "Generating HTML report..."
        mantra report --mantra-config ${mantraConfig} --formats=html output.html
      '';

      installPhase = ''
        mkdir -p $out
        cp output.html $out/
      '';
    };

  in
  {
    packages = {
      inherit mantra-db mantra-report;
    };
  };
}
