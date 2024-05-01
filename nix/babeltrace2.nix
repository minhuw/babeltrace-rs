{
  autoreconfHook,
  pkg-config,
  swig,
  gcc8Stdenv,
  fetchurl,
  glib,
  popt,
  elfutils,
  bison,
  flex,
  asciidoc,
  xmlto,
  docbook_xml_dtd_45,
  docbook_xsl,
  docbook_xsl_ns,
  ncurses,
  python3,
}:
gcc8Stdenv.mkDerivation rec {
  pname = "babeltrace";
  version = "2.0.5";

  src = fetchurl {
    url = "https://github.com/efficios/${pname}/archive/refs/tags/v${version}.tar.gz";
    sha256 = "sha256-wFj5otwLEofcwg71n8DhP1S5xDwU/9WNXcHBB4aISoE=";
  };

  # The pre-generated ./configure script uses an old autoconf version which
  # breaks cross-compilation (replaces references to malloc with rpl_malloc).
  # Re-generate with nixpkgs's autoconf. This requires glib to be present in
  # nativeBuildInputs for its m4 macros to be present.
  nativeBuildInputs = [
    autoreconfHook
    glib
    pkg-config
    swig
    python3
  ];
  buildInputs = [
    glib
    popt
    elfutils
    bison
    flex
    asciidoc
    xmlto
    docbook_xml_dtd_45
    docbook_xsl
    docbook_xsl_ns
    docbook_xml_dtd_45
    ncurses
    python3
  ];

  # --enable-debug-info (default) requires the configure script to run host
  # executables to determine the elfutils library version, which cannot be done
  # while cross compiling.
  configureFlags = [ "--enable-python-bindings" ]; # + lib.optional (stdenv.hostPlatform != stdenv.buildPlatform) "--disable-debug-info";
}
