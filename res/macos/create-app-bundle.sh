#!/bin/bash
set -e

target=$1

BUNDLE_VERSION="$(cargo pkgid | cut -d "#" -f2)"
BUNDLE_NAME="space-acres"
BUNDLE_BUILD=$(date +"%Y%m%d%H%M")

APP_PREFIX=target/bundle

function process_dependencies()
{
    local target=$1
    local destdir=$2
    local file=$3

    echo "Processing $file"

    # Intel x86_64 path
    local inst_prefix=/usr/local/*
    if [[ $target == "aarch64-apple-darwin" ]]; then
        # Arm64 path
        inst_prefix=/opt/homebrew/*
    fi

    local DEPS=$(dyld_info -dependents $file | tail -n +4)
    local process_list=""
    for dep in $DEPS; do
        if [[ $dep == $inst_prefix ]]; then
            dep_file=$(basename $dep)
            new_dep_file=$destdir/$dep_file
            if [ ! -f $new_dep_file ]; then
                # Not exist, do copy
                echo "  Copying $dep"
                cp -n $dep $destdir
            fi

            # Fix the dependency
            echo "  Patching $dep"
            install_name_tool -change $dep @executable_path/../Frameworks/lib/$dep_file $file

            # Collect list of dependencies
            process_list="$new_dep_file $process_list"
        fi
    done

    # Recursively process dependencies
    for dep in $process_list; do
        process_dependencies $target $destdir $dep
    done
}

function create_plist_file()
{
    local output_file=$1
    cat > $output_file <<EOL
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
<key>CFBundleDevelopmentRegion</key>
<string>English</string>
<key>CFBundleDisplayName</key>
<string>space-acres</string>
<key>CFBundleExecutable</key>
<string>space-acres</string>
<key>CFBundleIconFile</key>
<string>space-acres.icns</string>
<key>CFBundleIdentifier</key>
<string>network.subspace.space-acres</string>
<key>CFBundleInfoDictionaryVersion</key>
<string>6.0</string>
<key>CFBundleName</key>
<string>$BUNDLE_NAME</string>
<key>CFBundlePackageType</key>
<string>APPL</string>
<key>CFBundleShortVersionString</key>
<string>$BUNDLE_VERSION</string>
<key>CFBundleVersion</key>
<string>$BUNDLE_BUILD</string>
<key>CSResourcesFileMapped</key>
<true/>
<key>LSRequiresCarbon</key>
<true/>
<key>NSHighResolutionCapable</key>
<true/>
</dict>
</plist>
EOL
}

function sign_binary()
{
    local file=$1

    echo Signing $file

    codesign --force --verify -vvvv --sign - $file
}

# 1. Delete bundle if already exists
if [ -d "$APP_PREFIX/$target" ]; then
    rm -rf $APP_PREFIX/$target
fi

# 2. Create the bundle
mkdir -p $APP_PREFIX/$target/SpaceAcres.app/Contents/{MacOS,Resources,Frameworks}
mkdir -p $APP_PREFIX/$target/SpaceAcres.app/Contents/Frameworks/{lib,etc,share}
cp target/$target/production/space-acres $APP_PREFIX/$target/SpaceAcres.app/Contents/MacOS

cp res/macos/space-acres.icns $APP_PREFIX/$target/SpaceAcres.app/Contents/Resources

create_plist_file $APP_PREFIX/$target/SpaceAcres.app/Contents/Info.plist

# 3. Copy and fix dependencies
destDir=$APP_PREFIX/$target/SpaceAcres.app/Contents/Frameworks/lib
process_dependencies $target $destDir $APP_PREFIX/$target/SpaceAcres.app/Contents/MacOS/space-acres

for lib in $APP_PREFIX/$target/SpaceAcres.app/Contents/Frameworks/lib/*.dylib; do
    sign_binary $lib
done

sign_binary $APP_PREFIX/$target/SpaceAcres.app/Contents/MacOS/space-acres
