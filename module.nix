flake:
{
  config,
  lib,
  pkgs,
  ...
}:
with lib;

let
  service_name = "divera-status-tracker";
  cfg = config.services.${service_name};
  pkg = (flake.defaultPackage.${pkgs.stdenv.hostPlatform.system});
in
{
  options.services.${service_name} = {
    enable = mkEnableOption "enable ${service_name} services";
    config_path = mkOption {
      type = types.path;
      description = "The config path";
    };
    data_path = mkOption {
      type = types.path;
      description = "The data path";
    };
    timer = mkOption {
      type = types.str;
      description = "The timer value to set";
    };
  };

  config = mkIf cfg.enable {
    systemd.services."${service_name}" = {
      path = [ "${pkg.divera-status-tracker}" ];
      description = "Update the status data";
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "divera-status-tracker --config ${cfg.config_path} --data ${cfg.data_path} update";
        ProtectHome = "read-only";
      };
    };

    systemd.timers.${service_name} = {
      description = "${service_name} timer";
      wantedBy = [ "timers.target" ]; # Ensure the timer is activated at boot
      timerConfig = {
        OnCalendar = cfg.timer;
        Persistent = true; # Ensures the job runs after missed events (e.g., after reboot)
        Unit = "${service_name}.target";
      };
    };
  };
}
