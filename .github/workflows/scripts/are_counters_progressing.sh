#!/bin/bash

# This script verifies whether multiple counters output by an input command all progress
# over time (exit code 0) or not (exit code 1). The script ends when either of the following
# is reached first: all counters have been incremented at least once or the script has
# timed out.
# ---
# It expects a command that outputs any number of counters as hex strings, e.g.:
# ```
# "0x1"
# "0x2"
# "0x3"
# ```
# , a timeout in seconds, and an interval in seconds.
# The input command is executed at every interval to retrieve the latest counter values.
# ---
# Usage: ./are_counters_progressing.sh <command> <increment_timeout> <check_interval>

# Input parameters
command_to_run="$1"         # Command to execute
increment_timeout="$2"      # Timeout for increment (in seconds)
check_interval="$3"         # Interval to check (in seconds)

# Function to extract decimal value from hexadecimal string
extract_decimal() {
    without_quotes=${1//\"/}
    printf "%d" "$without_quotes"
}

# Execute the command once to get the initial outputs
initial_outputs=($(eval "$command_to_run" | tr -d '\r'))

# Loop through the initial outputs to initialize prev_outputs and incremented_flags
declare -a prev_outputs
declare -a incremented_flags
for output in "${initial_outputs[@]}"; do
    prev_outputs+=("$output")
    incremented_flags+=("false")
done

# Loop until all outputs have been incremented at least once within all iterations or timeout is reached
while true; do
    # Execute the command and get the current outputs
    current_outputs=($(eval "$command_to_run" | tr -d '\r'))

    # Check if all outputs have been incremented at least once
    all_incremented=true
    for ((i=0; i<${#current_outputs[@]}; i++)); do
        prev_output_decimal=$(extract_decimal "${prev_outputs[$i]}")
        current_output_decimal=$(extract_decimal "${current_outputs[$i]}")
        if [ "$current_output_decimal" -gt "$prev_output_decimal" ]; then
            incremented_flags[$i]="true"
        elif [ "${incremented_flags[$i]}" == "false" ]; then
            all_incremented=false
        fi
    done

    # Print current outputs with flags indicating whether they have been incremented
    for ((i=0; i<${#current_outputs[@]}; i++)); do
        if [ "${incremented_flags[$i]}" == "true" ]; then
            echo "${current_outputs[$i]} (✅Incremented)"
        else
            echo "${current_outputs[$i]}"
        fi
    done

    echo -e "\n"

    # Update prev_outputs with current_outputs for the next iteration
    prev_outputs=("${current_outputs[@]}")

    # Sleep for check_interval seconds
    sleep "$check_interval"

    # Check if all outputs have been incremented at least once and exit the loop
    if $all_incremented; then
        echo "All outputs incremented at least once"
        exit 0
    fi

    # Check if the timeout has been reached
    if [ "$SECONDS" -ge "$increment_timeout" ]; then

        kurtosis service logs cdk-v1 zkevm-agglayer-001
        echo "No increment within $increment_timeout seconds"
        exit 1
    fi
done
