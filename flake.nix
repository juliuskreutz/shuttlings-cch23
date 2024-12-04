{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        shuttle = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-shuttle";
          version = "0.49.0";

          doCheck = false;

          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-yRIRu0F4BiM/KfKTwty+uzH4uvhbiYd+XxzAQoxyb6o=";
          };

          cargoHash = "sha256-8F8iUprhSFB3LZBEl5XNObvhc301/akGpP/nI9YFZ+4=";
        };

        cch23-validator = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cch23-validator";
          version = "22.0.2";

          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-x+UgW2ExmPuyG8htyby7LtvOtF2zpGeS2k/CmQiJbJo=";
          };

          cargoHash = "sha256-C43X9wxnwnupgPTNqBoApeVj0ZsjNFv1XNA779otBws=";
        };
      in {
        devShells.default = with pkgs;
          mkShell {
            buildInputs = [
              rust-bin.stable.latest.default
              rust-analyzer
              taplo
              pkg-config
              openssl
              sqlx-cli
              shuttle
              cch23-validator
            ];
          };
      }
    );
}
