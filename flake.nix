{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    nix-filter.url = "github:numtide/nix-filter";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    nix-filter,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      system = "x86_64-linux";
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      libraries = with pkgs; [
        wayland
        pkg-config
        libGL
        dbus
        expat
        fontconfig
        freetype
        libxkbcommon
        libclang
        gobject-introspection
        gst_all_1.gstreamer
        gst_all_1.gst-plugins-base
        openssl
        alsa-lib
        ffmpeg
      ];
      rustToolchain = pkgs.rust-bin.beta.latest.default; # beta required due to anyhow requiring cargo above 1.83
      library_path = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" libraries;
      rust_platform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
      sickle_package = rust_platform.buildRustPackage {
        pname = "sickle";
        version = "0.1";
        # src = ./.;
        src = nix-filter.lib.filter {
          # root = ~/Documents/Programming/astrum_unstable;
          root = self;
          include = [
            "src"
            ./src
            ./Cargo.lock
            ./Cargo.toml
          ];
        };
        buildInputs = libraries;

        nativeBuildInputs = with pkgs; [
          pkg-config
          libclang
          makeBinaryWrapper
        ];

        RUSTFLAGS = map (a: "-C link-arg=${a}") [
          "-Wl,--push-state,--no-as-needed"
          "-lEGL"
          "-lwayland-client"
          "-Wl,--pop-state"
          "--release"
        ];

        postInstall =
          #bash
          ''
            cd ${self}
            install -Dm644 ../data/sickle.svg $out/share/icons/hicolor/scalable/apps/sickle.svg
            install -Dm644 ../data/sickle.png $out/share/icons/hicolor/32x32/apps/sickle.svg
            install -Dm644 ../data/sickle.desktop $out/share/applications/sickle.desktop
            install -Dm644 ../data/sickle-pause-symbolic.svg $out/share/icons/hicolor/scalable/actions/sickle-pause-symbolic.svg
            install -Dm644 ../data/sickle-play-symbolic.svg $out/share/icons/hicolor/scalable/actions/sickle-play-symbolic.svg
            install -Dm644 ../data/sickle-scissors-symbolic.svg $out/share/icons/hicolor/scalable/actions/sickle-scissors-symbolic.svg

                    wrapProgram "$out/bin/sickle"\
                      --prefix CARGO_MANIFEST_DIR : "${self}"\
                      --prefix LD_LIBRARY_PATH : ${
              pkgs.lib.makeLibraryPath (with pkgs; [
                libxkbcommon
                vulkan-loader
                xorg.libX11
                xorg.libXcursor
                xorg.libXi
              ])
            }

          '';

        verbose = true;
        doCheck = false;

        useFetchCargoVendor = true;

        cargoHash = "sha256-ckotvpQw3WfvJ2YXR/XKT7LamGj7kLtGwMR/qrXpmYc=";
        cargoLock = {
          lockFile = ./Cargo.lock;
          allowBuiltinFetchGit = true;
        };
      };
    in {
      packages.sickle = sickle_package;

      defaultPackage = self.packages.${system}.sickle;

      devShells.default = pkgs.mkShell {
        buildInputs = libraries;

        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (with pkgs; [
          wayland
          libGL
          libxkbcommon
          libclang
        ])}";
        GST_PLUGIN_PATH = "${pkgs.gst_all_1.gstreamer}:${pkgs.gst_all_1.gst-plugins-bad}:${pkgs.gst_all_1.gst-plugins-ugly}:${pkgs.gst_all_1.gst-plugins-good}:${pkgs.gst_all_1.gst-plugins-base}";
      };
    });
}
