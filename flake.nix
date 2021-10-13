{
  inputs = {
    nixCargoIntegration.url = "github:yusdacra/nix-cargo-integration";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = inputs: inputs.nixCargoIntegration.lib.makeOutputs {
    root = ./.;
  };
}
