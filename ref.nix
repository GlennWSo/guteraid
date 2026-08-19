{
  description = "flake for rust dev";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
        pkgsCross = import nixpkgs {
          overlays = [(import rust-overlay)];
          localSystem = system;
          crossSystem = {
            config = "x86_64-w64-mingw32";
            libc = "msvcrt";
          };
        };
        mkRust = p:
          p.rust-bin.stable.latest.default.override {
            targets = [
              "x86_64-pc-windows-gnu"
              "x86_64-unknown-linux-gnu"
            ];
          };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src"];
          targets = [
            "x86_64-pc-windows-gnu"
            "x86_64-unknown-linux-gnu"
          ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain (_p: rust);
        craneCross = (crane.mkLib pkgsCross).overrideToolchain mkRust;
        src = nixpkgs.lib.fileset.toSource rec {
          root = ./.;
          fileset = nixpkgs.lib.fileset.unions [
            (craneCross.fileset.commonCargoSources root)
            ./placeholders
          ];
        };

        commonRust = {
          inherit src;
          buildInputs = with pkgs; [
            # Add extra build inputs here, etc.
            openssl
            alsa-lib.dev
            udev.dev
            xorg.libX11.dev
            xorg.libXcursor.dev
            xorg.libXi.dev
            udev

            clang
            lld
          ];
          nativeBuildInputs = with pkgs; [
            # Add extra native build inputs here, etc.
            pkg-config
          ];
        };

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
          glib
          gtk3
          libxkbcommon
          libz
          pkg-config
          vulkan-loader
          wayland
          wayland-protocols
          zlib
          alsa-lib.dev
          udev.dev
          udev
          alsa-lib
        ]);

        windowsDepsOnly = craneCross.buildDepsOnly {
          inherit src;
          pname = "WindowsDeps";
        };
        cargoArtifacts = craneLib.buildDepsOnly (commonRust
          // {
            # Be warned that using `//` will not do a deep copy of nested sets
            pname = "mycrate-deps";
          });
      in rec {
        devShells.default = craneLib.devShell {
          inherit LD_LIBRARY_PATH;
          inputsFrom = [packages.bkad];
          packages = with pkgs; [
            rust-analyzer
            bacon
            cargo-insta
            wf-recorder
            wine64
          ];
        };
        packages = rec {
          default = bkad;
          bkad = craneLib.buildPackage (commonRust
            // {
              inherit cargoArtifacts;
            });

          winBkad = craneCross.buildPackage {
            # src = craneLib.cleanCargoSource ./.;
            inherit src;
            cargoArtifacts = windowsDepsOnly;
            strictDeps = true;
          };
        };
      }
    );
}
