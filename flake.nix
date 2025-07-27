{
  description = "A CLI tool to prompt users for confirmation before running potentially unsafe kubectl commands.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages = rec {
          kubectl-check = pkgs.rustPlatform.buildRustPackage {
            pname = "kubectl-check";
            version = "0.2440.0";
            src = ./.;

            cargoBuildFlags = "-p kubectl-check";

            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };
          default = kubectl-check;
        };
      }
    );
}
