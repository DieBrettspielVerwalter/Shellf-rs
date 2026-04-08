{ self, inputs, ... }:
{
  perSystem = { system, pkgs, self', craneLib, craneCommonArgs, src, migrations_src, ... }:
  let
    grant = ./grant.sql;
    migratedDbTemplate = pkgs.stdenv.mkDerivation {
      name = "mariadb-migrated-template";
      src = migrations_src;         
      nativeBuildInputs = [ pkgs.mariadb pkgs.sqlx-cli ];

      buildPhase = ''
        export MYSQL_BASE_DIR=$TMPDIR/mysql
        mkdir -p $MYSQL_BASE_DIR/data
        mariadb-install-db --datadir=$MYSQL_BASE_DIR/data --auth-root-authentication-method=normal --skip-test-db > /dev/null

        # Server im Hintergrund starten
        mariadbd --datadir=$MYSQL_BASE_DIR/data --socket=$MYSQL_BASE_DIR/mysql.sock --skip-networking &
        until mariadb-admin --socket=$MYSQL_BASE_DIR/mysql.sock ping > /dev/null 2>&1; do sleep 1; done

        mariadb --socket=$MYSQL_BASE_DIR/mysql.sock -u root -e "CREATE DATABASE SpieleDB;"
        export DATABASE_URL="mysql://root@localhost/SpieleDB?socket=$MYSQL_BASE_DIR/mysql.sock"
        
        # Migrationen ausführen (setzt voraus, dass /migrations im src ist)
        sqlx migrate run --source $src

        mariadb --socket=$MYSQL_BASE_DIR/mysql.sock -u root -e "source ${grant}"
        echo "sourced"
        cat ${grant}
        mariadb --socket=$MYSQL_BASE_DIR/mysql.sock -u root -e "SELECT * FROM SpieleDB.Spielkopie;"

        mariadb-admin --socket=$MYSQL_BASE_DIR/mysql.sock -u root shutdown
      '';

      installPhase = "cp -r $MYSQL_BASE_DIR/data $out";
    };
  in
  {
    packages.migratedDbTemplate = migratedDbTemplate;
  };
}