{
  description = "flake for rust dev";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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
        rust = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain (_p: rust);
        src = nixpkgs.lib.fileset.toSource rec {
          root = ./.;
          fileset = nixpkgs.lib.fileset.unions [
            (craneLib.fileset.commonCargoSources root)
            # ./assets
          ];
        };

        buildInputs = with pkgs; [
          # Add extra build inputs hxere, etc.
          openssl
          alsa-lib
          libdecor
          libxkbcommon
          udev
          vulkan-loader
          wayland
          wayland-protocols
        ];
        commonRust = {
          inherit src buildInputs;
          nativeBuildInputs = with pkgs; [
            # Add extra native build inputs here, etc.
            pkg-config
            # makeWrapper
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonRust
          // {
            # Be warned that using `//` will not do a deep copy of nested sets
            pname = "mycrate-deps";
          });
      in rec {
        devShells.default = craneLib.devShell {
          # inherit LD_LIBRARY_PATH;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          inputsFrom = [packages.bkad];
          packages = with pkgs; [
            rust-analyzer
            bacon
            cargo-insta
            wf-recorder
            wine64
            pinta
          ];
        };
        packages = rec {
          default = bkad;
          deps = cargoArtifacts;
          bkad = craneLib.buildPackage (commonRust
            // {
              inherit cargoArtifacts;
              nativeBuildInputs = with pkgs; [
                # Add extra native build inputs here, etc.
                pkg-config
                makeWrapper
                autoPatchelfHook
              ];

              # postInstall= ''
              postInstall = ''
                # echo "hello world" > $out/hello.txt
                mkdir $out/lib
                cp target/release/libbevy_dylib.so $out/lib/
              '';
              # postFixup = ''
              #   wrapProgram $out/bin/guteraid \
              #     --prefix LD_LIBRARY_PATH : "$out/lib"
              # '';
            });
        };
      }
    );
}
