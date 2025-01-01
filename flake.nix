# nix develop to run
# or nix develop --impure -c $SHELL
{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; };
      toolDeps = with pkgs; [
        pkg-config
        cmake
        python3
        clang
        protobuf
      ];
      runtimeDeps = with pkgs; [
        expat
        freetype
        fontconfig
        vulkan-validation-layers
      ];
      libraryDeps =
        with pkgs;
        with xorg;
        [
          alsa-lib
          brotli
          bzip2
          libpng
          libX11
          libXcursor
          libXi
          libXrandr
          openssl
          zstd
          vulkan-loader
          vulkan-validation-layers
          wayland
          libxkbcommon
        ];
      libPath = pkgs.lib.makeLibraryPath libraryDeps;
    in
    {
      devShells.x86_64-linux.default = pkgs.mkShell {
        packages = toolDeps ++ runtimeDeps ++ libraryDeps;

        # Set environment variables
        LD_LIBRARY_PATH = "${libPath}:/run/opengl-driver/lib";
        PROTOC = "${pkgs.protobuf}/bin/protoc";
      };
      packages = rec {
        loungy = pkgs.rustPlatform.buildRustPackage {
          name = "loungy";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
        };
        default = loungy;
      };

      # You could also define extra shells or packages here
    };
}
