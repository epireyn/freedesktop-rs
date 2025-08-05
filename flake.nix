{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-analyzer"
                "clippy"
                "rust-src"
                "rustfmt"
                "cargo"
                "rust-std"
              ];
            })
          ];
        };
      });
}
