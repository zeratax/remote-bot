{
  description = "Rust managed by rustup";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
      libPath = pkgs.lib.makeLibraryPath [
        # load external libraries that you need in your rust project here
      ];
    in
    {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          clang
          llvmPackages.bintools
          rustup
          pkg-config
          openssl
        ];

        RUSTC_VERSION = overrides.toolchain.channel;

        LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

        shellHook = ''
          export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
          export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
          export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig:$PKG_CONFIG_PATH
        '';

        RUSTFLAGS = builtins.map (a: ''-L ${a}/lib'') [
          # add libraries here (e.g. pkgs.libvmi)
        ];

        LD_LIBRARY_PATH = libPath;

        BINDGEN_EXTRA_CLANG_ARGS =
        (builtins.map (a: ''-I"${a}/include"'') [
          pkgs.glibc.dev
          # add dev libraries here (e.g. pkgs.libvmi.dev)
        ]) ++ [
          ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
          ''-I"${pkgs.glib.dev}/include/glib-2.0"''
          ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
        ];
      };
    }
  );
}
