{
  inputs = {
    pkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, utils, ... }@inputs:
    utils.lib.eachDefaultSystem (system:
      let
        overlays = [ inputs.rust-overlay.overlay ];
        pkgs = import inputs.pkgs { inherit system overlays; };

        # Get the latest rust nightly
        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override {
            extensions = [ "rust-src" "rust-analyzer-preview" ];
          });

      in {
        # `nix develop`
        devShell = pkgs.mkShell rec {
          # supply the specific rust version
          nativeBuildInputs = [ pkgs.gcc pkgs.openssl rust pkgs.pkg-config ];
          RUST_SRC_PATH = "${rust}";
        };
      });
}
