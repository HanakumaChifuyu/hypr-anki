{
  description = "Rust + Slint development environment for hypranki";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
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
          # Winit loads some windowing and graphics libraries at runtime.
          linuxLibraries = lib.optionals pkgs.stdenv.hostPlatform.isLinux (
            with pkgs;
            [
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
            name = "hypranki-rust-slint";

            packages = with pkgs; [
              fish
              rustc
              cargo
              clippy
              rustfmt
              rust-analyzer
              cargo-generate
              slint-lsp
              pkg-config
              cmake
              slint-viewer
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
