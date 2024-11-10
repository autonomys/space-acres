For each target, the script does these steps:
1) Build the project for that target
2) Create folder structure for MacOS app bundle
3) Copy dependencies from system
4) Fix rpath of the files in bundle to make sure it has no external dependencies
5) Sign the final output

After finishing all individual targets, it combines into a single universal bundle and create dmg file for that bundle
