## Tabry Nix Configurations

### Try it out without installing

```
nix run github:evanbattaglia/tabry-rs --help

# e.g.:
# source <(nix run github:evanbattaglia/tabry-rs bash)
# or 
# nix run github:evanbattaglia/tabry-rs fish | source
```

### Use in a flake (e.g. Home Manager package list)


```
{
  inputs = {
    tabry.url = "github:evanbattaglia/tabry-rs";
  };
  outputs = { ..., tabry }: {
     ....
     config.home.packages = [
       ....
       tabry.packages.${system}.default
    ]
  }
}
```

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
