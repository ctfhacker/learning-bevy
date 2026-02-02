{
  description = "Bevy CLI flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay"; # Nightly toolchain pinning.
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Pinned nightly with extra components and WASM target.
        rustToolchain = pkgs.rust-bin.nightly."2026-01-22".default.override {
          extensions = [ "rust-analyzer" "clippy" "rustfmt" "rust-src" "rustc-dev" "llvm-tools" ];
          targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown" ];
        };

        # Build bevy_lint with the same nightly to satisfy rustc-private deps.
        nightlyRustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        # Bevy CLI from main branch.
        bevy-cli = pkgs.rustPlatform.buildRustPackage {
          pname = "bevy-cli";
          version = "main";
          doCheck = false;
          src = pkgs.fetchFromGitHub {
            owner = "TheBevyFlock";
            repo = "bevy_cli";
            rev = "main";
            hash = "sha256-Swj7j/A7Mgd2ufSADZdGMXOLbmvpdHGJfQVFCaWX9yg=";
          };
          cargoHash = "sha256-0r5rzUkdho0PDscCionjySdiG/6d5sv+Nv/4tfEMzP8=";
          meta = with pkgs.lib; {
            description = "Bevy CLI";
            homepage = "https://github.com/TheBevyFlock/bevy_cli";
            license = licenses.mit;
          };
        };

        # bevy_lint + bevy_lint_driver from bevy_cli/bevy_lint.
        bevy-lint = nightlyRustPlatform.buildRustPackage rec {
          pname = "bevy-lint";
          version = "main";
          doCheck = false;
          src = pkgs.fetchFromGitHub {
            owner = "TheBevyFlock";
            repo = "bevy_cli";
            rev = "main";
            hash = "sha256-Swj7j/A7Mgd2ufSADZdGMXOLbmvpdHGJfQVFCaWX9yg=";
          };
          # bevy_lint expects a local Cargo.lock.
          prePatch = ''
            cp Cargo.lock bevy_lint/Cargo.lock
          '';
          cargoRoot = "bevy_lint";
          cargoLock = { lockFile = "${src}/Cargo.lock"; };
          cargoBuildFlags = [
            "-p"
            "bevy_lint"
            "--bin"
            "bevy_lint"
            "--bin"
            "bevy_lint_driver"
          ];
          meta = with pkgs.lib; {
            description = "Bevy Lint";
            homepage = "https://github.com/TheBevyFlock/bevy_cli/tree/main/bevy_lint";
            license = licenses.mit;
          };
        };
      in {
        packages = {
          bevy-cli = bevy-cli;
          bevy_lint = bevy-lint;
          cargo-generate = pkgs.cargo-generate;
          rust-toolchain = rustToolchain;
          default = bevy-cli;
        };
        devShells.default = pkgs.mkShell {
          packages = [
            bevy-cli
            bevy-lint
            rustToolchain
            pkgs.cargo-generate
            pkgs.pkg-config
            pkgs.rustup
            pkgs.alsa-lib
            pkgs.binaryen
            pkgs.systemd
            pkgs.wasm-bindgen-cli
            pkgs.wayland
          ];
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
            pkgs.alsa-lib
            pkgs.systemd
            pkgs.wayland
          ];

          shellHook = ''
            if [ -f Cargo.toml ] && bevy lint -- --version >/dev/null 2>&1; then
              echo "bevy lint: OK (installed)"
            else
              echo "warning: 'bevy lint' failed in dev shell; ensure bevy_lint is on PATH"
            fi
          '';
        };
      });
}
