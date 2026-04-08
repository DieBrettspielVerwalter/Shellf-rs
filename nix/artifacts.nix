{...}:
{
  perSystem = { craneLib, craneCommonArgs, pkgs, self', src, emptyCargoArtifacts, ... }:
  let
    cargoArtifacts = craneLib.buildDepsOnly craneCommonArgs;

    llvmCovArtifacts = craneLib.mkCargoDerivation (craneCommonArgs // {
      src = craneLib.mkDummySrc { src = src; };

      pnameSuffix = "-llvm-cov-deps";
      
      cargoArtifacts = null;
      doInstallCargoArtifacts = true;   # Export the deps (target)

      env = (craneCommonArgs.env or { }) // {
        # Export a marker variable in case any scripts or hooks want to customize
        # how they run depending on if they are running here or with the "real"
        # project sources.
        # NB: *just* in case someone tries to set this to something specific, honor it
        CRANE_BUILD_DEPS_ONLY = 1;
      };

      nativeBuildInputs = with pkgs; [ cargo-llvm-cov openssl pkg-config ];
      buildPhaseCargoCommand = "cargoWithProfile llvm-cov --locked --all-targets test";
    });
  in
  {
    _module.args = {
      cargoArtifacts = cargoArtifacts;
      llvmArtifacts  = llvmCovArtifacts;
    };
  };
}