{...}:
{
  perSystem = { craneLib, craneCommonArgs, cargoArtifacts, ... }:
  let
    shellfClippy = craneLib.cargoClippy (craneCommonArgs // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--all-targets -- --deny warnings --deny missing_docs";
    });
  in {
    checks = {
      inherit shellfClippy;
    };
  };
}