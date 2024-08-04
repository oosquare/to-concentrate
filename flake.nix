{
  description = "Rust application environment.";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem = { self', inputs', system, ... }: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ (import inputs.rust-overlay) ];
        };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-analyzer" "rust-src" ]; 
        };
      in {
        packages.tomatify = pkgs.callPackage ./pkgs/tomatify {};
        packages.default = self'.packages.tomatify;

        devShells.default = let
          mkShell = pkgs.mkShell.override { stdenv = pkgs.stdenvNoCC; };
        in
          mkShell {
            buildInputs = [ rust ];
          };
      };
    };
}
