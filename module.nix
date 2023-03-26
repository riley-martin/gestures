{ config, lib, pkgs, ... }:
with lib;
let
  cfg = config.services.gestures;

in {
  options = {
    services.gestures = {
      enable = mkEnableOption "${pkgName}";
      gestureList = mkOption {
        default = [];
        description = "List of gestures to match";
        type = with types; listOf ( submodule {
          swipe = mkOption {
            type = types.attrsOf ( submodule {
              direction = mkOption {
                type = types.str;
                default = "any";
                description = "direction of gesture";
              };
              fingers = mkOption {
                type = types.int;
                description = "number of fingers in the gesture";
              };
              start = mkOption {
                type = types.string;
                description = "Command executed on gesture start";
                default = "";
              };
              update = mkOption {
                type = types.string;
                description = "Command executed on gesture update";
                default = "";
              };
              end = mkOption {
                type = types.string;
                description = "Command executed on gesture end";
                default = "";
              };
            });
          };
        });
      };
    };
  };

  config = mkIf cfg.enable {
    
  };
}