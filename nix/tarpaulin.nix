{...}:
{
  perSystem = { craneLib, craneCommonArgs, cargoArtifacts, ... }:
  let
    shellfTarpaulin = craneLib.cargoTarpaulin (craneCommonArgs // {
      inherit cargoArtifacts;
      cargoTarpaulinExtraArgs = "--all-targets --all-features --out Html --output-dir $out";
    });
  in {
    checks = {
      inherit shellfTarpaulin;
    };
  };
}