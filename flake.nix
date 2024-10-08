{
  description = "A Discord Bot to Remotely Control Different Things";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
        overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
        libPath = pkgs.lib.makeLibraryPath [
          # load external libraries that you need in your rust project here
        ];
      in {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            clang
            llvmPackages.bintools
            rustup
            pkg-config
            openssl
          ];

          RUSTC_VERSION = overrides.toolchain.channel;

          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [pkgs.llvmPackages_latest.libclang.lib];

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
            ])
            ++ [
              ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
              ''-I"${pkgs.glib.dev}/include/glib-2.0"''
              ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
            ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "remote-bot";
          version = "0.1.0";

          src = builtins.path {
            path = ./.;
            name = "remote-bot";
          };

          cargoHash = "sha256-I+uStfLJxwgyrT1RACFalMB1bgd8HPEjlzi2qBJ77Jw=";

          nativeBuildInputs = [pkgs.openssl pkgs.pkg-config];
          buildInputs = [pkgs.openssl.dev];
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/remote-bot";
        };
      }
    )
    // {
      nixosModules = {
        default = {
          lib,
          config,
          pkgs,
          ...
        }: let
          workingDir = "/var/lib/remote-bot";
          settingsFile = "${workingDir}/settings.toml";
          cfg = config.services.remote-bot;
        in {
          options.services.remote-bot = {
            enable = lib.mkEnableOption "Enable the remote-bot service.";

            settings = {
              discordToken = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "Discord token";
              };
              recipientEmail = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "Recipient email";
              };
              senderDomain = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "Sender domain";
              };
              smtpPassword = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "SMTP password";
              };
              smtpServer = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "SMTP server";
              };
              smtpUsername = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "SMTP username";
              };
              timezone = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "Timezone";
              };
              envFile = lib.mkOption {
                type = lib.types.nullOr lib.types.path;
                default = null;
                description = "Path to env file containing secrets.";
              };
            };
          };

          config = lib.mkIf cfg.enable {
            assertions = [
              {
                assertion =
                  lib.any (v: v != null) [
                    cfg.settings.discordToken
                    cfg.settings.recipientEmail
                    cfg.settings.senderDomain
                    cfg.settings.smtpPassword
                    cfg.settings.smtpServer
                    cfg.settings.smtpUsername
                    cfg.settings.timezone
                  ]
                  || cfg.settings.envFile != null;
                message = "All options must be set unless an envFile is specified.";
              }
            ];

            systemd.services.remote-bot = {
              description = "Remote Bot Service";
              after = ["network.target"];
              wantedBy = ["multi-user.target"];
              serviceConfig =
                {
                  WorkingDirectory = workingDir;
                  ExecStartPre = ''
                    mkdir -p ${workingDir}
                    cat <<EOF > ${settingsFile}
                    # Generate settings based on provided options
                    ${
                      if cfg.settings.discordToken != null
                      then "discord_token=${cfg.settings.discordToken}"
                      else ""
                    }
                    ${
                      if cfg.settings.recipientEmail != null
                      then "recipient_email=${cfg.settings.recipientEmail}"
                      else ""
                    }
                    ${
                      if cfg.settings.senderDomain != null
                      then "sender_domain=${cfg.settings.senderDomain}"
                      else ""
                    }
                    ${
                      if cfg.settings.smtpPassword != null
                      then "smtp_password=${cfg.settings.smtpPassword}"
                      else ""
                    }
                    ${
                      if cfg.settings.smtpServer != null
                      then "smtp_server=${cfg.settings.smtpServer}"
                      else ""
                    }
                    ${
                      if cfg.settings.smtpUsername != null
                      then "smtp_username=${cfg.settings.smtpUsername}"
                      else ""
                    }
                    ${
                      if cfg.settings.timezone != null
                      then "timezone=${cfg.settings.timezone}"
                      else ""
                    }
                    EOF
                  '';
                  ExecStart = "${self.packages.${pkgs.system}.default}/bin/remote-bot";
                }
                // lib.optionalAttrs (cfg.settings.envFile != null) {EnvironmentFile = cfg.settings.envFile;};
            };
          };
        };
      };
    };
}
