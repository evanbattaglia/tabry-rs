## Tabry Nix Configurations

### Home Manager Module

This repository provides a home-manager (https://github.com/nix-community/home-manager)
module to make tabry easy to install and use via home manager.

To use the home-manager module via flakes, add this module to your home-manager
configuration:

```nix
{
  inputs = {
    tabry.url = "github:evanbattaglia/tabry-rs";
  };
  outputs = { ..., tabry }: {
    homeConfigurations.<user> = {
      modules = [
        tabry.homeModules.tabry
        {
          config.programs.tabry = {
            enable = true;
            enableBashIntegration = true;
            tabryFiles = [
              ./zellij.tabry
            ];
          };
        }
      ]
    }
  };
}
```
