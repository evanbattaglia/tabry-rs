{ config, lib, pkgs, ... }:

# This file contains a Home Manager module that installs tabry
# and sets up tabry configuration files to be used by tabry
#
let

  cfg = config.programs.tabry;

  tabry = pkgs.callPackage ../default.nix {};
  tabryLang = pkgs.callPackage ./tabry-lang.nix { inherit tabry; };

  # converts /nix/store/.../foo.tabry to "foo"
  commandNameFromTabryFilename = fileName: 
    (builtins.replaceStrings [".tabry"] [""] (builtins.baseNameOf fileName));

  mkInitFish = fileName: let
    commandName = commandNameFromTabryFilename fileName;
  in ''
    tabry_completion_init ${commandName}
  '';

  compileTabryFiles = map tabryLang.compileTabryFile;

in {

  options.programs.tabry = {
    enable = lib.mkEnableOption "tabry, a tab completion library";
    enableFishIntegration = lib.mkEnableOption "enables fish completions";
    enableBashIntegration = lib.mkEnableOption "enables bash completions";
    tabryFiles = lib.mkOption {
      type = with lib.types; listOf path;
      default = [];
      description = ''
        *.tabry files to be compiled to completion json
      '';
    };
  };

  config = lib.mkIf cfg.enable (
    let
      tabryImportsPath = builtins.concatStringsSep ":" (compileTabryFiles cfg.tabryFiles);
    in {
      home.packages = [tabry];

      # for each file, compile it to json
      # then add the dir to $TABRY_IMPORT_PATH

      programs.fish.shellInit = lib.mkIf cfg.enableFishIntegration (
        ''
          set -x TABRY_IMPORT_PATH "${tabryImportsPath}:$TABRY_IMPORT_PATH"
          ${tabry}/bin/tabry fish | source
          ${builtins.concatStringsSep "\n" (map mkInitFish cfg.tabryFiles)}
        ''
      );

      programs.bash.initExtra = lib.mkIf cfg.enableBashIntegration (
        ''
          set -x TABRY_IMPORT_PATH "${tabryImportsPath}:$TABRY_IMPORT_PATH"
          source <(${tabry}/bin/tabry bash)
        ''
      );
    }
  );
}