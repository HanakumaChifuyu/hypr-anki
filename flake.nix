{
  description = "Rust + Tauri + React development environment for hypranki";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

      perSystem =
        { pkgs, ... }:
        let
          inherit (pkgs) lib;
          # Tauri uses WebKitGTK and GTK on Linux.
          linuxLibraries = lib.optionals pkgs.stdenv.hostPlatform.isLinux (
            with pkgs;
            [
              gtk3
              webkitgtk_4_1
              libsoup_3
              glib
              librsvg
              openssl
              libayatana-appindicator
              fontconfig
              freetype
              libGL
              libxkbcommon
              libx11
              libxcb
              libxcursor
              libxi
              libxrandr
              vulkan-loader
              wayland
            ]
          );
        in
        {
          formatter = pkgs.nixfmt;

          devShells.default = pkgs.mkShell {
            name = "hypranki";

            packages = with pkgs; [
              fish
              rustc
              cargo
              clippy
              rustfmt
              rust-analyzer
              cargo-generate
              sqlx-cli
              nodejs_24
              cargo-tauri
              pkg-config
              cmake
            ];

            buildInputs = linuxLibraries;

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

            # macOS uses native frameworks supplied by the Nix stdenv SDK.
            shellHook =
              lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
                export LD_LIBRARY_PATH="${lib.makeLibraryPath linuxLibraries}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
              ''
              + ''
                # Use fish for interactive sessions; keep nix develop -c working.
                case $- in
                  *i*)
                    export SHELL="${pkgs.fish}/bin/fish"
                    exec "$SHELL"
                    ;;
                esac
              '';
          };
        };
    };
}
