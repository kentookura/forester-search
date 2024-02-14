{
  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    forester.url = "sourcehut:~jonsterling/ocaml-forester";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, forester }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        libraries = with pkgs; [ ];

        packages = with pkgs; [ ];
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "forest-search";
          version = "0.2.2";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = libraries;
        };
        devShell = pkgs.mkShell {
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = with pkgs;
            libraries ++ [
              forester.packages.${system}.default
              texlive.combined.scheme-full
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" "rust-analyzer-preview" "rustfmt" ];
              })

            ];
        };
      });
}
