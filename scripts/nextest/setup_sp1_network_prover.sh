#!/bin/sh

# Exit with 1 if NEXTEST_ENV isn't defined.
if [ -z "$NEXTEST_ENV" ]; then
    exit 1
fi

# echo "SP1_PRIVATE_KEY=" >> "$NEXTEST_ENV"
