{
  description = "Toolchain setup for Magnet Manipulation";

  inputs = {
    your-nixos-flake.url = "github:maxkiv/nix";
    nixpkgs.follows = "your-nixos-flake/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    ros2 = {
      url = "github:MaxKiv/nix-ros-overlay?ref=master";
      # type = "path";
      # path = "/home/max/git/nix-ros-overlay";
      inputs.nixpkgs.follows = "your-nixos-flake/nixpkgs";
    };
    poetry2nix.url = "github:nix-community/poetry2nix";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    ros2,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [
        ros2.overlays.default
      ];

      pkgs = import nixpkgs {
        inherit system overlays;
        config.allowUnfree = true;
      };

      rosJazzyPkgs = pkgs.rosPackages.jazzy;

      rosPkgs = with rosJazzyPkgs; [
        ros-core
        ros2cli
        ros2launch
        rclcpp
        std-msgs
        can-msgs
        sensor-msgs

        ament-cmake
        ament-cmake-core
        ament-index-python
        ament-lint
        ament-package
        can-msgs
        diagnostic-updater
        rclcpp
        ros-core
        ros2-control
        ros2cli
        ros2launch
        std-msgs
        yaml-cpp-vendor
        canopen-interfaces
      ];

      # Get a cross compilation toolchain from the rust-toolchain.toml
      toolchain = with fenix.packages.${system};
        fromToolchainFile {
          file = ./rust-toolchain.toml; # alternatively, dir = ./.;
          sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
          # sha256 = pkgs.lib.fakeSha256;
        };

      poetry2nix = inputs.poetry2nix.lib.mkPoetry2Nix {inherit pkgs;};

      # Use poetry2nix to create a Python environment from pyproject.toml
      pythonEnv = poetry2nix.mkPoetryEnv {
        projectDir = ./.;
        preferWheels = true;
      };
    in {
      devShells = {
        nanocli = pkgs.mkShell {
          __structuredAttrs = true; # Add this line

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
        default = pkgs.mkShell rec {
          RUST_BACKTRACE = "full";

          nativeBuildInputs = with pkgs; [
            clang
            llvmPackages.libclang
          ];

          buildInputs =
            (with pkgs; [
              nil
              alejandra
              toolchain
              # rust-analyzer

              # ros crap
              colcon

              openssl
              pkg-config

              # GUI libs
              libxkbcommon
              libGL
              fontconfig

              # wayland libraries
              wayland
              wayland-protocols

              egl-wayland # provides libEGL with wayland platform support
              libglvnd # libGLVND -- helps load vendor GL provider
              mesa # mesa core

              # x11 libraries
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              xorg.libX11
              pythonEnv
              poetry
            ])
            ++ rosPkgs;

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";

          WINIT_UNIX_BACKEND = "wayland";

          # shellHook = ''
          #   export CMAKE_PREFIX_PATH=${pkgs.lib.makeSearchPath "share" rosPkgs}:$CMAKE_PREFIX_PATH
          # '';

          shellHook = ''
            echo "ROS_DISTRO = $ROS_DISTRO"
            echo "AMENT_PREFIX_PATH = $AMENT_PREFIX_PATH"

            export LIBCLANG_PATH=${pkgs.llvmPackages.libclang.lib}/lib
            echo "Using libclang from $LIBCLANG_PATH"

            # Add glibc headers to clang’s search path
            export GLIBC_INCLUDE_PATH=${pkgs.stdenv.cc.libc_dev}/include

            echo "Using sysroot: ${pkgs.stdenv.cc.libc_dev}"
            echo "Using glibc headers from: $GLIBC_INCLUDE_PATH"
          '';
        };
      };
    });
  nixConfig = {
    extra-substituters = ["https://ros.cachix.org"];
    extra-trusted-public-keys = ["ros.cachix.org-1:dSyZxI8geDCJrwgvCOHDoAfOm5sV1wCPjBkKL+38Rvo="];
  };
}
