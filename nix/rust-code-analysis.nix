{...}:
{
  perSystem = { craneLib, craneCommonArgs, cargoArtifacts, src, pkgs, ... }:
  let
    rustCodeAnalysis = "${pkgs.rust-code-analysis}/bin/rust-code-analysis-cli";

    emptyCargoArtifacts = pkgs.stdenv.mkDerivation {
      name = "empty-cargo-artifacts";
      buildCommand = "mkdir -p $out";  # does nothing
    };

    # Derivation to produce the metrics txt file
    rustMetrics = craneLib.mkCargoDerivation (craneCommonArgs // {
      # inherit cargoArtifacts;
      cargoArtifacts = emptyCargoArtifacts;
      doInstallCargoArtifacts = false;
      pname = "rust-metrics";
      version = "0.1.0";

      buildPhaseCargoCommand = ''
        # Run rust-code-analysis-cli twice and compute percentage
        val1=$(${rustCodeAnalysis} -O json --metrics -p ${src} \
                | ${pkgs.jq}/bin/jq '.. | objects | select(.kind=="function")' \
                | ${pkgs.jq}/bin/jq -s '[.[] | select(has("name"))] | length')

        val2=$(${rustCodeAnalysis} -O json --metrics -p ${src} \
                | ${pkgs.jq}/bin/jq '.. | objects | select(.kind=="function" and .metrics.cyclomatic.sum > 10)' \
                | ${pkgs.jq}/bin/jq -s '[.[] | select(has("name"))] | length')

        perc=0
        if [ "$val1" -ne 0 ]; then
          perc=$((100 * val2 / val1))
        fi

        mkdir $out
        echo -e "$val1\n$val2\n$perc%" > $out/stats.txt
      '';
    });
  in
  {
    _module.args.emptyCargoArtifacts = emptyCargoArtifacts;
    packages = {
      inherit rustMetrics;
    };
  };
}
