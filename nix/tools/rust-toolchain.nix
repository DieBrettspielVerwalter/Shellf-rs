{ self, inputs, ... }:
{
  flake = {
    overlays.rustToolchain = final: prev: {
      rustToolchain =
        prev.rust-bin.nightly."2025-12-15".default.override {
          extensions = [ "rust-src" "rust-docs" /*"llvm-tools-preview"*/ ];
        };
    };
  };
}
