{ config, lib, pkgs, ... }:

let
  cfg = config.services.to-concentrate;
  toml = pkgs.formats.toml {};
in {
  options.services.to-concentrate = {
    enable = lib.mkEnableOption "to-concentrate";

    package = lib.mkPackageOption pkgs "to-concentrate" {};

    settings = lib.mkOption {
      type = toml.type;
      description = "Settings for to-concentrate daemon";

      default = {
        duration = {
          preparation = 900;
          concentration = 2400;
          relaxation = 600;
        };

        notification = {
          preparation = {
            summary = "Preparation Stage End";
            body = "It's time to start concentrating on learning.";
          };

          concentration = {
            summary = "Concentration Stage End";
            body = "Well done! Remember to have a rest.";
          };

          relaxation = {
            summary = "Relaxation Stage End";
            body = "Feel energetic now? Let's continue.";
          };
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile."to-concentrate/config.toml".source =
      toml.generate "to-concentrate-config.toml" cfg.settings;

    systemd.user.services.to-concentrate-daemon = {
      Unit = {
        Description = "To Concentrate daemon";
      };

      Service = {
        Type = "exec";
        ExecStart = "${cfg.package}/bin/to-concentrate-daemon";
      };

      Install = {
        WantedBy = [ "default.target" ];
      };
    };
  };
}
