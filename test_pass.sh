#!/bin/bash

generate_password() {
    #matrix=$1
    random_indices=("${@:1}")  # Takes all arguments starting from the second as an array
    #if [ -z "$matrix" ]; then
        matrix=0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz
    #fi

    # Initialize the password variable
    pass=""

    # Generate password using provided random indices
    for index in "${random_indices[@]}"; do
        char_index=$((index % ${#matrix}))  # Calculate matrix index
        pass="$pass${matrix:$char_index:1}"
    done

    echo "$pass"
}

generate_password ${@:1}

#FQmAzH
