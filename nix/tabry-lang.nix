{ stdenv, tabry, ... }:
  let
    # converts /nix/store/.../foo.tabry to "foo"
    commandNameFromTabryFilename = filename: 
      (builtins.replaceStrings [".tabry"] [""] (builtins.baseNameOf filename));

    formatJsonFilename = tabryFilename: 
      (commandNameFromTabryFilename tabryFilename) + ".json";

    # This is a function that takes a .tabry file
    # and returns a derivation that compiles that
    # .tabry file into the tabry .json file
    compileTabryFile = inFile: stdenv.mkDerivation {
      name = "tabry-compile-file-${inFile}";
      buildPhase = ''
        mkdir $out
        ${tabry}/bin/tabry compile < ${inFile} > $out/${formatJsonFilename inFile}
      '';
      # by default, stdenv.mkDerivation will run `make install`
      # which we don't want to do here
      dontInstall = true;
      dontUnpack = true;
    };

  in {
    inherit compileTabryFile commandNameFromTabryFilename;
  }
