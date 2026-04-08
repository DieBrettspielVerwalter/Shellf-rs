{ inputs, ... }:
{
  perSystem = { pkgs, lib, system, ... }:
  let
    pkgs' = import inputs.nixpkgs {
      inherit system;
      overlays = [
        inputs.rust-overlay.overlays.default
        (final: prev: {
          rustToolchain =
            prev.rust-bin.nightly."2025-12-15".default.override {
              extensions = [ "rust-src" "rust-docs" "llvm-tools-preview" ];
            };
        })
      ];
    };
  in
  {
    devShells.default = pkgs.mkShell {
      name = "rust-dev-shell";

      buildInputs = with pkgs'; [
        # Nutzt die Toolchain aus dem Overlay
        rustToolchain
        
        # Häufig benötigte Helfer für Rust-Entwicklung
        rust-analyzer
        cargo-edit
        cargo-watch

        cargo-llvm-cov

        openssl
        pkg-config
      ];

      shellHook = ''
        echo "🦀 Rust Dev-Shell geladen (Nightly 2025-12-15)"
        cargo --version
      '';
    };
  };
}