{
  description = "Full development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        libPath = with pkgs;
          lib.makeLibraryPath [
            systemd
            openssl
          ];
      in {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            pkg-config
            systemd
            openssl
            cmake
          ];
          buildInputs = with pkgs; [
            clang
            llvmPackages.bintools
            rustup
            bash
            yaml-language-server

            libudev-zero
            udev
            alsa-lib-with-plugins
            wayland
            libxkbcommon
            vulkan-loader
            vulkan-tools
            pipewire
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr

            zed-editor
          ];

          RUSTC_VERSION = "nightly";

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [pkgs.llvmPackages_latest.libclang.lib];

          shellHook = ''
            export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
            export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
          '';

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs)}:/run/opengl-driver/lib:/run/opengl-driver-32/lib";
        };
      }
    );
}
