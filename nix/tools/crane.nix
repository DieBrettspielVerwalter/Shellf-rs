{ inputs, ... }:
{
  perSystem = { system, pkgs, src, lib, ... }:
    let
      pkgs' = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlays.default ];
      };

      buildLlvmCovDepsOnly = args: craneLib.mkCargoDerivation (args // {
        pnameSuffix = "-llvm-cov-deps";
        nativeBuildInputs = (args.nativeBuildInputs or [ ]) ++ [ pkgs.cargo-llvm-cov ];
        buildPhaseCargoCommand = "cargo llvm-cov --locked --all-targets --no-run";
        doInstallCargoArtifacts = true;
        doCheck = false;
      });

      craneLib =
        (inputs.crane.mkLib pkgs').overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "llvm-tools-preview" ];
          }
        );

      commonArgs = {
        inherit src;
        strictDeps = true;

        # Tools, die zur Build-Zeit laufen (pkg-config)
        nativeBuildInputs = [ 
          pkgs'.pkg-config 
        ];

        # Bibliotheken, gegen die gelinkt wird (openssl)
        buildInputs = [ 
          pkgs'.openssl 
        ];

        # Wichtig: Verhindert, dass das openssl-sys crate versucht, 
        # eine eigene statische OpenSSL-Version zu bauen
        OPENSSL_NO_VENDOR = 1;
        SQLX_OFFLINE = "true";
        LLVM_COV_TARGET_DIR = "target";

        # MariaDB deterministic paths
        MYSQL_DATADIR     = "/build/source/mysql/data";
        MYSQL_UNIX_PORT   = "/build/source/mysql/mariadb.sock";
        DATABASE_URL      = "mysql://root@localhost/SpieleDB?socket=/build/source/mysql/mariadb.sock";
      };

      sqlFilter = path: _type: builtins.match ".*sql$" path != null;
      sqlxFilter = path: _type: builtins.match ".*json$" path != null;
      customOrCargo = path: type:
        (sqlFilter path type) || (sqlxFilter path type) || (craneLib.filterCargoSources path type);
    in
    {
      # expose for reuse in other modules
      _module.args.craneLib = craneLib;
      _module.args.craneCommonArgs = commonArgs;
      _module.args.src = lib.cleanSourceWith {
        src = ./../..;
        filter = customOrCargo;
        name = "source";
      };
      _module.args.migrations_src = lib.cleanSourceWith {
        src = ./../../migrations;
        filter = sqlFilter;
        name = "migrations";
      };
    };
}