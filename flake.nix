{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixCargoIntegration.url = "github:yusdacra/nix-cargo-integration";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = inputs:
    let pkgs = import inputs.nixpkgs { system = "x86_64-linux"; };
    in inputs.nixCargoIntegration.lib.makeOutputs {
      root = ./.;
      overrides.shell = common: prev: {
        packages = prev.packages ++ [
          (pkgs.writeScriptBin "update-readme" ''
            cargo readme --no-indent-headings -o README.md
          '')
        ];
      };
    };
}
