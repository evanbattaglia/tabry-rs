{lib, callPackage, stdenv, installShellFiles, pkgs}:
let 
  tabry = pkgs.callPackage ../default.nix {};
  tabryLang = pkgs.callPackage ./tabry-lang.nix { inherit tabry; };
in {

  withTabry =
    tabryFile: package: 
      let
        cmd = builtins.replaceStrings [".tabry"] [""] (builtins.baseNameOf tabryFile);
        compiledTabryFile = tabryLang.compileTabryFile tabryFile;
        formatPackageName = name: lib.toUpper(builtins.replaceStrings ["-" "."] ["_" "_"] name);
      in stdenv.mkDerivation {
        name = "${package.name}-with-tabry";
        nativeBuildInputs = [ installShellFiles ];
        src = ./.;
        installPhase = ''
          mkdir -p $out/bin
          cp -R ${package}/bin $out/

          ${tabry}/bin/tabry bash ${compiledTabryFile} \
            --uniq-fn-id _NIX_${formatPackageName package.name} >> ${cmd}.bash

          installShellCompletion ${cmd}.bash

          ${tabry}/bin/tabry zsh ${compiledTabryFile} \
            --uniq-fn-id _NIX_${formatPackageName package.name} >> ${cmd}.zsh

          installShellCompletion ${cmd}.zsh

          ${tabry}/bin/tabry fish ${compiledTabryFile} \
            --uniq-fn-id _NIX_${formatPackageName package.name} >> ${cmd}.fish

          installShellCompletion ${cmd}.fish
        '';
      };
}
