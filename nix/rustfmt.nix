{...}:
{
  perSystem = { craneLib, craneCommonArgs, cargoArtifacts, ... }:
  let
    shellfFmt = craneLib.cargoFmt (craneCommonArgs // {
      inherit cargoArtifacts;
      # optional extra args for cargo fmt (workspace mode)
      cargoExtraArgs = "--all --check";
      # optional extra args for rustfmt itself
      rustFmtExtraArgs = "";
    });
  in {
    checks = {
      inherit shellfFmt;
    };
  };
}