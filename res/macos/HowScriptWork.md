For each target, the script do these steps:
1) Build the project for that target
2) Create folder structure for MacOS app bundle
3) Copy dependencies from system
4) Fix rpath of the files in bundle to make sure it has no external dependencies
5) Sign the final output

After finish all individual targets, it combine into a single universal bundle and create dmg file for that bundle


This function do copy the dependency and path the executable/lib
```bash
function process_dependencies()
```

This function create Info.plist file for MacOS app bundle:
```bash
function create_plist_file()
```

This function signs binaries:
```bash
function sign_binary()
```