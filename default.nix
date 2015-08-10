with import <nixpkgs> {}; with xlibs;
  
  stdenv.mkDerivation rec {
      name = "wtftw";
      buildInputs = [ makeWrapper cargo rustcMaster libXinerama libX11 ];
      buildPhase = "cargo build";
      libPath = lib.makeLibraryPath [ libXinerama libX11 ];
      unpackPhase = "true";
  }
