{
  stdenv,
  autoconf,
  automake,
  bison,
  callPackage,
  cmake,
  elfutils,
  rustPlatform,
  flex,
  glib,
  libtool,
  libelf,
  pcre2,
  pkg-config,
}:
let
  babeltrace2 = callPackage ./babeltrace2.nix { };
  db_plugin = rustPlatform.buildRustPackage { 
    pname = "db_converter";
    version = "0.0.1";

    src = ../.;

    buildInputs = [
      babeltrace2
      glib
      pcre2
    ];

    nativeBuildInputs = [
      pkg-config
      autoconf
      automake
      rustPlatform.bindgenHook
    ];

    docheck = false;

    cargoLock = {
      lockFile = ../Cargo.lock;
    };
  };
in
stdenv.mkDerivation {
  pname = "babeltrace2_converter_plugin";
  version = "0.0.1";

  src = ../.;

  nativeBuildInputs = [
    cmake
    babeltrace2
    bison
    pkg-config
    glib
    libtool
    flex
  ];

  buildInputs = [
    elfutils
    glib
    libelf
    db_plugin
  ];

  NIX_BUILD = true;

  docheck = false;
}
