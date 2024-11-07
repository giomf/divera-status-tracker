{
  description = "Divera status tracker flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      naersk,
    }:
    {
      nixosModules.default = import ./module.nix self;
    }
    // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        naersk' = pkgs.callPackage naersk { };
      in
      rec {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };
        overlays.default = final: prev: { divera-status-tracker = defaultPackage; };
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            clippy
            rustc
            rust-analyzer
            rustfmt
          ];
          env = {
            POLARS_FMT_MAX_ROWS = 50;
            # POLARS_FMT_MAX_COLS = 50;
          };
        };
      }
    );
}
