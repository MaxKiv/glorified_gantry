{
  description = "Nanotec nanolib example CLI build environment";

  inputs = {
    your-nixos-flake.url = "github:maxkiv/nix";
    nixpkgs.follows = "your-nixos-flake/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};

      # Get a cross compilation toolchain from the rust-toolchain.toml
      toolchain = with fenix.packages.${system};
        fromToolchainFile {
          file = ./rust-toolchain.toml; # alternatively, dir = ./.;
          sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
          # sha256 = pkgs.lib.fakeSha256;
        };
    in {
      devShells = {
        nanocli = pkgs.mkShell {
          buildInputs = with pkgs; [
            gcc
            gnumake
            pkg-config
            libcap # for setcap
            util-linux # often useful
          ];

          # LD_LIBRARY_PATH so the linker/runtime finds nanolib .so files
          shellHook = ''
            export LD_LIBRARY_PATH=$PWD/vendor/nanolib_cpp_linux_1.4.0/nanotec_nanolib/lib/:$LD_LIBRARY_PATH
            echo "DevShell ready. Run 'make' to build the example."
          '';
        };

        # Development shells provided by this flake, to use:
        # nix develop .#default
        default = pkgs.mkShell {
          RUST_BACKTRACE = "full";

          buildInputs = with pkgs; [
            nil # Nix LSP
            alejandra # Nix Formatter
            toolchain # Our Rust toolchain
            rust-analyzer # Rust LSP
          ];
        };
      };

      packages.default = pkgs.stdenv.mkDerivation {
        pname = "nanolib-cli";
        version = "1.0";

        src = ./vendor/nanolib_cpp_linux_1.4.0/NanolibExample;

        nativeBuildInputs = [pkgs.makeWrapper];

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
