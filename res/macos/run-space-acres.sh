#!/bin/bash

cd "$(dirname "$0")"
EXECUTABLE_PATH="$(pwd)"

export GDK_PIXBUF_MODULE_FILE="$EXECUTABLE_PATH/../Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"

exec "$EXECUTABLE_PATH/space-acres"