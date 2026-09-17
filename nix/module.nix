{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.atuin-web;
in
{
  options.services.atuin-web = {
    enable = lib.mkEnableOption "atuin-web, a browser UI for an atuin sync server";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.atuin-web;
      defaultText = lib.literalExpression "pkgs.atuin-web";
      description = "The atuin-web package to run.";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = ''
        Address to listen on. The default keeps atuin-web on loopback, for a
        deployment that puts a reverse proxy in front of it.
      '';
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "Port to listen on.";
    };

    atuinServerUrl = lib.mkOption {
      type = lib.types.str;
      default = "http://127.0.0.1:8888";
      example = "https://atuin.example.com";
      description = "Base URL of the upstream atuin sync server.";
    };

    sessionExpiry = lib.mkOption {
      type = lib.types.int;
      default = 86400;
      description = "How long a browser session stays valid, in seconds.";
    };

    logLevel = lib.mkOption {
      type = lib.types.enum [
        "trace"
        "debug"
        "info"
        "warn"
        "error"
      ];
      default = "info";
      description = "Log level.";
    };

    secureCookies = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Set the Secure flag on session cookies. Enable this whenever the site is
        served over HTTPS; leaving it off there lets the cookie travel in clear.
      '';
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Open {option}`services.atuin-web.port` in the firewall.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      example = "/run/secrets/atuin-web";
      description = ''
        Environment file holding settings that must not reach the world-readable
        Nix store, read by systemd as `EnvironmentFile`. `ATUIN_WEB_TOKEN` is the
        one that matters: an atuin session token, which signs every visitor in as
        that account and so removes the login prompt.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.atuin-web = {
      description = "atuin-web, a browser UI for an atuin sync server";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];

      environment = {
        ATUIN_WEB_BIND = "${cfg.host}:${toString cfg.port}";
        ATUIN_WEB_SERVER_URL = cfg.atuinServerUrl;
        ATUIN_WEB_SESSION_EXPIRY = toString cfg.sessionExpiry;
        ATUIN_WEB_LOG_LEVEL = cfg.logLevel;
        ATUIN_WEB_SECURE_COOKIES = lib.boolToString cfg.secureCookies;
      };

      serviceConfig = {
        ExecStart = lib.getExe cfg.package;
        EnvironmentFile = lib.mkIf (cfg.environmentFile != null) [ cfg.environmentFile ];
        DynamicUser = true;
        Restart = "on-failure";
        RestartSec = 5;

        CapabilityBoundingSet = "";
        DevicePolicy = "closed";
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        PrivateUsers = true;
        ProcSubset = "pid";
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectProc = "invisible";
        ProtectSystem = "strict";
        RemoveIPC = true;
        RestrictAddressFamilies = [
          "AF_INET"
          "AF_INET6"
        ];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [
          "@system-service"
          "~@privileged"
        ];
        UMask = "0077";
      };
    };

    networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [ cfg.port ];
  };
}
