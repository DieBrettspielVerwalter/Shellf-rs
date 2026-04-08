{
  perSystem = { pkgs, craneLib, craneCommonArgs, cargoArtifacts, llvmArtifacts, self', ... }:
  let
    makeCoverageFragment = { testName, isBinary }: 
    let
      safeName = pkgs.lib.replaceStrings [ "::" ] [ "_" ] testName;

      preBuildDB = ''
        # Create deterministic MariaDB directories
        mkdir -p "$MYSQL_DATADIR"
        echo "$MYSQL_DATADIR"
        echo "$PWD"
        cp -r ${self'.packages.migratedDbTemplate}/* "$MYSQL_DATADIR/"
        chmod -R +w "$MYSQL_DATADIR"

        # Start MariaDB
        mariadbd --datadir="$MYSQL_DATADIR" --socket="$MYSQL_UNIX_PORT" --skip-networking &
        until mysqladmin --socket="$MYSQL_UNIX_PORT" ping > /dev/null 2>&1; do sleep 1; done
      '';

      postBuildTransform = ''
        # cat test.log
        # echo $PWD
        # ls -al
        mkdir -p $out
        # RAW_JSON_LINES=$(cat test.log | sed -r 's/\x1B\[[0-9;]*[mK]//g' | grep "mantra: req-id=" | sed -nE "s/.*req-id=\`([^\`]+)\`; file='([^']+)'; line='([0-9]+)';.*/{\"req_id\":\"\1\", \"file\":\"\2\", \"line\":\3}/p" || true)
        RAW_JSON_LINES=$(tr -d '\r' < test.log \
          | sed -r 's/\x1B\[[0-9;]*[mK]//g' \
          | grep "mantra: req-id=" \
          | sed -nE "s/.*req-id=\`([^\`]+)\`; file='([^']+)'; line='([0-9]+)';.*/{\"req_id\":\"\1\",\"file\":\"\2\",\"line\":\3}/p" || true)
        if [ -z "$RAW_JSON_LINES" ]; then
            TRACES_JSON="[]"
        else
            TRACES_JSON=$(echo "$RAW_JSON_LINES" | jq -s '.')
        fi
        TEST_LOG_CONTENT=$(cat test.log)
        jq --arg test_name "${testName}" \
            --argjson traces "$TRACES_JSON" \
            --arg pwd "$PWD/" \
            --arg logs "$TEST_LOG_CONTENT" \
            -f ${../tools/coverage/transform_to_coverage_schema.jq} coverage.json > $out/fragment.json
      '';

      extraLlvmCovArgs = "--no-clean --json --output-path coverage.json ${if isBinary then "--all-targets" else ""} -- ${testName} --nocapture 2>&1 | tee test.log";

    in craneLib.mkCargoDerivation (craneCommonArgs // {
      cargoArtifacts = llvmArtifacts;
      pnameSuffix = "-fragment-${safeName}";

      nativeBuildInputs = (craneCommonArgs.nativeBuildInputs or [ ]) ++ (with pkgs; [ cargo-llvm-cov openssl pkg-config mariadb jq ]);

      preBuild = preBuildDB;
      postBuild = postBuildTransform;

      buildPhaseCargoCommand = ''
        ls /build/source/mysql/
        echo $DATABASE_URL
        cargoWithProfile llvm-cov --locked ${if isBinary then "--all-targets" else ""} test --json --output-path coverage.json -- "${testName}" --nocapture 2>&1 | tee test.log
      '';
      installPhaseCargoCommand = ''
        mkdir -p $out
        # cp report.json $out/report.json
      '';

      installPhaseCommand = "mkdir -p $out"; 
      doInstallCargoArtifacts = false;
    });

    testNames = [
      "application::use_cases::capture_game::tests::capture_game_use_case_executes_and_saves_game"
      "application::use_cases::capture_game::tests::capture_game_use_case_fails_when_game_save_fails"
      "application::use_cases::capture_game::tests::capture_game_use_case_handles_empty_optional_fields"
      "application::use_cases::create_player::tests::create_player_use_case_executes_and_saves_player"
      "application::use_cases::create_player::tests::create_player_use_case_fails_when_save_fails"
      "application::use_cases::create_player::tests::edit_player_use_case_executes_update_correctly"
      "application::use_cases::create_player::tests::edit_player_use_case_fails_when_update_fails"
      "application::use_cases::lend_game::tests::lend_game_use_case_executes_successfully"
      "application::use_cases::lend_game::tests::lend_game_use_case_fails_when_lend_copy_fails"
      "application::use_cases::lend_game::tests::lend_game_use_case_fails_with_invalid_copy_id"
      "application::use_cases::plan_game_night::tests::plan_game_night_use_case_allows_empty_participants_and_copies"
      "application::use_cases::plan_game_night::tests::plan_game_night_use_case_propagates_save_failure"
      "application::use_cases::plan_game_night::tests::plan_game_night_use_case_saves_game_night_successfully"
      "application::use_cases::record_game_session::tests::record_game_session_use_case_allows_none_game_night_and_empty_participants"
      "application::use_cases::record_game_session::tests::record_game_session_use_case_fails_with_invalid_copy_id"
      "application::use_cases::record_game_session::tests::record_game_session_use_case_propagates_save_failure"
      "application::use_cases::record_game_session::tests::record_game_session_use_case_saves_session_successfully"
      "application::use_cases::return_game::tests::return_game_use_case_calls_repository_successfully"
      "application::use_cases::return_game::tests::return_game_use_case_propagates_repository_error"
      "infrastructure::repositories::sql::game_copy_repository::tests::sql__test__all__returns_all_saved_copies"
      "infrastructure::repositories::sql::game_copy_repository::tests::sql__test__delete_removes_copy"
      "infrastructure::repositories::sql::game_copy_repository::tests::sql__test__get_by_id__not_found"
      "infrastructure::repositories::sql::game_copy_repository::tests::sql__test__lend_and_return_copy"
      "infrastructure::repositories::sql::game_copy_repository::tests::sql__test__save_and_get_by_id"
      "infrastructure::repositories::sql::game_night_repository::tests::sql__test__save_and_all"
      "infrastructure::repositories::sql::game_repository::tests::sql__test__all__returns_all_saved_games"
      "infrastructure::repositories::sql::game_repository::tests::sql__test__get_by_name__not_found"
      "infrastructure::repositories::sql::game_repository::tests::sql__test__save_and_get_by_name"
      "infrastructure::repositories::sql::game_session_repository::tests::sql__test__save_and_all"
      "infrastructure::repositories::sql::player_repository::tests::sql__test__all__returns_all_saved_players"
      "infrastructure::repositories::sql::player_repository::tests::sql__test__get_by_email__not_found"
      "infrastructure::repositories::sql::player_repository::tests::sql__test__save_and_get_by_email"
    ];

    testBinaries = [
      # "menu_tests::check_main_menu_first"
      # "menu_tests::check_main_menu_second"
      # "menu_tests::check_main_menu_third"
      # "function_tests::check_neues_spiel_erstellen"
      # "function_tests::check_neues_spiel_erstellen_bgg_error"
      # "function_tests::check_spielkopie_anlegen"
      # "function_tests::check_spiel_verleihen"
      # "function_tests::check_spieleliste_anzeigen"
      # "function_tests::check_game_night_planning"
      # "function_tests::check_main_menu_second"

      "exit_enter"
      "help_enter"
      "mg_add_game_copy"
      "mg_back"
      "mg_capture_new_game"
      "mg_delete_game"
      "mg_delete_game_copy"
      "mg_edit_game"
      "mg_edit_game_copy"
      "mg_show_game_list"
      "mg_show_shelf"
      "pl_back"
      "pl_edit_player"
      "pl_lend_game"
      "pl_new_player"
      "pl_return_game"
      "pl_show_lending"
      "pl_show_players"
      "ps_back"
      "ps_delete_session"
      "ps_edit_session"
      "ps_plan_new"
      "ps_record_session"
      "ps_scheduled_nights"
      "ps_show_history"
      "ps_show_my_shelf"
    ];

    fragments_unittests = map (n: makeCoverageFragment { 
      testName = n;
      isBinary = false;
    }) testNames;

    fragments_binaries = map (n: makeCoverageFragment { 
      testName = n;
      isBinary = true;
    }) testBinaries;

    fragments = fragments_unittests ++ fragments_binaries;

  in {
    packages = let
      # Hilfsfunktion zum Erstellen der Attribute aus einer Liste von Namen und Drvs
      mkPkgMap = names: frags: 
        builtins.listToAttrs (
          map (i: {
            name = "test_${builtins.elemAt names i}";
            value = builtins.elemAt frags i;
          }) (builtins.genList (i: i) (builtins.length names))
        );

      unitTestPkgs = mkPkgMap testNames fragments_unittests;
      binaryPkgs   = mkPkgMap testBinaries fragments_binaries;
    in
      unitTestPkgs // binaryPkgs // {
        final-coverage = pkgs.stdenv.mkDerivation {
          name = "final-coverage-report.json";
          nativeBuildInputs = [ pkgs.jq ];
          srcs = fragments; 
          dontUnpack = true;
          installPhase = ''
            mkdir -p $out
            # Erstelle ein Array aus den Pfaden zu den einzelnen fragment.json Dateien
            files=()
            for s in $srcs; do
              if [ -f "$s/fragment.json" ]; then
                files+=("$s/fragment.json")
              fi
            done

            # jq mit der Liste der Dateien füttern
            jq -s -f ${../tools/coverage/merge_fragments.jq} "''${files[@]}" > $out/coverage.json
          '';
        };
      };
  };
}