{
  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = { self, nixpkgs, devenv, systems, ... } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (system: {
        devenv-up = self.devShells.${system}.default.config.procfileScript;
      });

      devShells = forEachSystem
        (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
            lib = pkgs.lib;
            stdenv = pkgs.stdenv;
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  # https://devenv.sh/reference/options/
                  packages = with pkgs; [
                    hello
                    glib
                    elfutils
                    flex
                    libcap
                    (hiPrio gcc)
                    llvmPackages_15.clangUseLLVM
                    llvmPackages_15.libllvm
                    llvmPackages_15.libclang
                  ];

                  # From: https://github.com/NixOS/nixpkgs/blob/1fab95f5190d087e66a3502481e34e15d62090aa/pkgs/applications/  networking/browsers/firefox/common.nix#L247-L253
                  # Set C flags for Rust's bindgen program. Unlike ordinary C
                  # compilation, bindgen does not invoke $CC directly. Instead it
                  # uses LLVM's libclang. To make sure all necessary flags are
                  # included we need to look in a few places.
                  enterShell = ''
                    export LD_LIBRARY_PATH=''${LD_LIBRARY_PATH%:}
                    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='sudo -E'
                    export LIBCLANG_PATH="${pkgs.llvmPackages_15.libclang.lib}/lib"
                    export BINDGEN_EXTRA_CLANG_ARGS="$(< ${stdenv.cc}/nix-support/libc-crt1-cflags) \
                      $(< ${stdenv.cc}/nix-support/libc-cflags) \
                      $(< ${stdenv.cc}/nix-support/cc-cflags) \
                      $(< ${stdenv.cc}/nix-support/libcxx-cxxflags) \
                      ${
                        lib.optionalString stdenv.cc.isClang
                        "-idirafter ${stdenv.cc.cc}/lib/clang/${
                          lib.getVersion stdenv.cc.cc
                        }/include"
                      } \
                      ${
                        lib.optionalString stdenv.cc.isGNU
                        "-isystem ${stdenv.cc.cc}/include/c++/${
                          lib.getVersion stdenv.cc.cc
                        } -isystem ${stdenv.cc.cc}/include/c++/${
                          lib.getVersion stdenv.cc.cc
                        }/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${
                          lib.getVersion stdenv.cc.cc
                        }/include"
                      } \
                    "
                  '';
                }
              ];
            };
          }
        );
    };
}
