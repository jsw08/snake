{
  description = "A devShell example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
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
      in {
        devShells.default = with pkgs;
          mkShell {
            buildInputs = [
              (
                pkgs.rust-bin.stable.latest.rust.override {
                  extensions = ["rust-src"];
                }
              )
            ];
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            nativeBuildInputs = with pkgs; [
              cargo
              rust-analyzer
              rustc
              gdb
            ];
          };
      }
    );
}
