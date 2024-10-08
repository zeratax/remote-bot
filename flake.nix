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
        lib = pkgs.lib;
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

          src = lib.sourceFilesBySuffices ./. [
            "Cargo.lock"
            "Cargo.toml"
            ".rs"
          ];

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

            environmentFile = lib.mkOption {
              type = lib.types.nullOr lib.types.path;
              default = null;
              description = "Path to an environment file to keep secrets out of the nix store.";
            };

            settings = {
              discord_token = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "Discord token";
              };
              recipient_email = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "Recipient email";
              };
              sender_domain = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "Sender domain";
              };
              smtp_password = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "SMTP password";
              };
              smtp_server = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "SMTP server";
              };
              smtp_username = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "SMTP username";
              };
              timezone = lib.mkOption {
                type = lib.types.nullOr lib.types.str;
                default = null;
                description = "Timezone";
              };
            };
          };

          config = lib.mkIf cfg.enable {
            assertions = [
              {
                assertion =
                  lib.any (v: v != null) [
                    cfg.settings.discord_token
                    cfg.settings.recipient_email
                    cfg.settings.sender_domain
                    cfg.settings.smtp_password
                    cfg.settings.smtp_server
                    cfg.settings.smtp_username
                    cfg.settings.timezone
                  ]
                  || cfg.environmentFile != null;
                message = "All options must be set unless an environment file is specified.";
              }
            ];

            users.users.remote-bot = {
              description = "User for remote-bot service";
              group = "remote-bot";
              home = workingDir;
              createHome = true;
              isSystemUser = true;
            };
            users.groups.remote-bot = {};

            systemd.services.remote-bot = {
              description = "Remote Bot Service";
              after = ["network.target"];
              wantedBy = ["multi-user.target"];

              serviceConfig =
                {
                  Type = "simple";
                  User = config.users.users.remote-bot.name;
                  Group = config.users.users.remote-bot.group;
                  WorkingDirectory = workingDir;

                  ExecStart = "${self.packages.${pkgs.system}.default}/bin/remote-bot";
                  Restart = "on-failure";

                  # Security Hardening
                  CapabilityBoundingSet = [""];
                  LockPersonality = true;
                  NoNewPrivileges = true;
                  PrivateDevices = true;
                  PrivateTmp = true;
                  ProcSubset = "pid";
                  ProtectClock = true;
                  ProtectControlGroups = true;
                  ProtectHome = true;
                  ProtectHostname = true;
                  ProtectKernelLogs = true;
                  ProtectKernelModules = true;
                  ProtectKernelTunables = true;
                  ProtectSystem = "strict";
                  ReadWritePaths = [workingDir];
                  RemoveIPC = true;
                  RestrictAddressFamilies = ["AF_INET" "AF_INET6" "AF_UNIX"];
                  RestrictNamespaces = true;
                  RestrictRealtime = true;
                  RestrictSUIDSGID = true;
                  SystemCallArchitectures = "native";
                  SystemCallFilter = ["@system-service" "~@resources" "~@privileged"];
                }
                // lib.optionalAttrs (cfg.environmentFile != null) {EnvironmentFile = cfg.environmentFile;};

              preStart = let
                format = pkgs.formats.toml {};
                settingsWithOutNull = lib.filterAttrsRecursive (name: value: value != null) cfg.settings;
                config = format.generate "settings.toml" settingsWithOutNull;
              in ''
                mkdir -p ${workingDir}
                ln -sf ${config} ${settingsFile}
              '';
            };
          };
        };
      };
    };
}
