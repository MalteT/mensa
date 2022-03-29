{
  inputs = {
    nixCargoIntegration.url = "github:yusdacra/nix-cargo-integration?ref=release-1.0";
  };

  outputs = inputs: inputs.nixCargoIntegration.lib.makeOutputs { root = ./.; };
}
