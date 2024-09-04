{
  description = "Tabry in Rust";
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = { nixpkgs, flake-utils, ... }:
    let
      tabryHmModule = import ./nix/tabry-hm-module.nix;
    in flake-utils.lib.eachDefaultSystem (
      system:
        let
          pkgs = nixpkgs.legacyPackages."${system}";
          tabry = pkgs.callPackage ./default.nix {};
        in {
          packages = {
            default = tabry;
          };
          devShell = pkgs.mkShell {
            packages = with pkgs; [
              rustfmt
              cargo
              rustc
            ];
          };
        }
    ) // {
      homeModules = {
        tabry = tabryHmModule;
      };
    };
}
