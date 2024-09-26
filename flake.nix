{
  description = "Tabry in Rust";
  inputs = {
    devshell.url = "github:numtide/devshell";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = { nixpkgs, flake-utils, devshell, ... }:
    let
      tabryHmModule = import ./nix/tabry-hm-module.nix;
    in flake-utils.lib.eachDefaultSystem (
      system:
        let
          pkgs = nixpkgs.legacyPackages."${system}";
          tabry = pkgs.callPackage ./default.nix {};
          tabryHelpers = pkgs.callPackage ./nix/tabry-helpers.nix {};
          foo = (tabryHelpers.withTabry ./docker/fish/foo.tabry (pkgs.writeShellScriptBin "foo" ''echo "Hello, world!"''));
        in {
          packages = {
            default = tabry;
            foo = foo;
          };
          devShell = pkgs.mkShell {
            packages = with pkgs; [
              rustfmt
              cargo
              rustc
            ];
          };
          devShells =
            let
              devShellpkgs = import nixpkgs {
                inherit system;
                overlays = [ devshell.overlays.default ];
              };
            in {
              test = devShellpkgs.devshell.mkShell {
                packages = [foo];
              };
              zsh = pkgs.mkShell {
                packages = [foo];
                shellHook = ''
                  ${pkgs.zsh}/bin/zsh
                '';
              };
              bash = pkgs.mkShell {
                packages = [foo];
              };
              fish = pkgs.mkShell {
                packages = [foo];
                shellHook = ''
                  ${pkgs.fish}/bin/fish
                '';
              };
            };
          helpers = tabryHelpers;
        }
    ) // {
      homeModules = {
        tabry = tabryHmModule;
      };
    };
}
