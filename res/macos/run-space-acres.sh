#!/bin/bash

cd "$(dirname "$0")"
LAUNCH_DIR="$(pwd)"

loaders=/tmp/saloaders.cache
if [ $loaders ]; then
  rm $loaders
fi

cp ../Frameworks/loaders.cache $loaders
sed -i '' "s|PREFIX|$(pwd)/../Frameworks|g" $loaders

export GDK_PIXBUF_MODULEDIR="$LAUNCH_DIR/../Frameworks/loaders"
export GDK_PIXBUF_MODULE_FILE=$loaders

"$LAUNCH_DIR/space-acres"