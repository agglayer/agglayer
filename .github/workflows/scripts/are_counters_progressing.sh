#!/bin/bash

# Usage: ./monitor_command.sh <command> <increment_timeout> <check_interval>

# Input parameters
command_to_run="$1"         # Command to execute
increment_timeout="$2"      # Timeout for increment (in seconds)
check_interval="$3"         # Interval to check (in seconds)

# Function to extract decimal value from hexadecimal string
extract_decimal() {
    without_quotes=${1//\"/}
    printf "%d" "$without_quotes"
}

# Set initial value
prev_output=$(eval "$command_to_run" | tr -d '\r' | tr -d '\n') # Remove carriage return and newline characters

# Parse the decimal value from the output
prev_output_decimal=$(extract_decimal "$prev_output")

# Loop until timeout
while sleep "$check_interval"; do
    current_output=$(eval "$command_to_run" | tr -d '\r' | tr -d '\n')
    current_output_decimal=$(extract_decimal "$current_output")

    # Check if the output has incremented
    echo $current_output $current_output_decimal
    if [ "$current_output_decimal" -gt "$prev_output_decimal" ]; then
        echo "Output incremented: $prev_output -> $current_output"
        exit 0
    fi

    # Check if the timeout has been reached
    if [ "$SECONDS" -ge "$increment_timeout" ]; then
        echo "No increment within $increment_timeout seconds"
        exit 1
    fi

    prev_output_decimal="$current_output_decimal"
done
