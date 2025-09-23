
{
  description = "Nanotec nanolib example CLI build environment";

  inputs = {
    your-nixos-flake.url = "github:maxkiv/nix";
    nixpkgs.follows = "your-nixos-flake/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }@ inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

      in {
        devShells.nanocli = pkgs.mkShell {
          buildInputs = with pkgs; [
            gcc
            gnumake
            pkg-config
            libcap          # for setcap
            util-linux      # often useful
          ];

          # LD_LIBRARY_PATH so the linker/runtime finds nanolib .so files
          shellHook = ''
            export LD_LIBRARY_PATH=$PWD/vendor/nanolib_cpp_linux_1.4.0/nanotec_nanolib/lib/:$LD_LIBRARY_PATH
            echo "DevShell ready. Run 'make' to build the example."
          '';
        };

        packages.default = pkgs.stdenv.mkDerivation {
          pname = "nanolib-cli";
          version = "1.0";

          src = ./vendor/nanolib_cpp_linux_1.4.0/NanolibExample;

          nativeBuildInputs = [ pkgs.makeWrapper ];

          buildPhase = ''
            make
          '';

          installPhase = ''
            mkdir -p $out/bin $out/lib
            cp bin/example $out/bin/
            cp ${./vendor/nanolib_cpp_linux_1.4.0/nanotec_nanolib/lib}/*.so $out/lib/
            wrapProgram $out/bin/example \
              --prefix LD_LIBRARY_PATH : ${pkgs.gcc.cc.lib}/lib \
              --prefix LD_LIBRARY_PATH : $out/lib \
          '';
        };
      });
}
