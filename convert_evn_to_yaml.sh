#!/bin/bash

# Start the YAML file with the required key
echo "env_variables:" >env.yaml

# Read the .env file line by line
while IFS='=' read -r key value; do
  # Skip empty lines and comments
  if [[ -n "$key" && ! "$key" =~ ^# ]]; then
    # Remove leading/trailing whitespace and quotes from the value
    value=$(echo "$value" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//;s/^"\(.*\)"$/\1/')
    # Add the key-value pair to the YAML file
    echo "  $key: \"$value\"" >>env.yaml
  fi
done <.env
