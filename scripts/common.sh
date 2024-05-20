check_unstaged_changes() {
    # Confirm we don't have unstaged changes
    if ! git diff --exit-code > /dev/null; then
        echo "Error: There are unstaged changes. Please commit or stash them before running this script."
        exit 1
    fi
}

confirm_root_directory() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo "Error: This script must be run from the root of a Git repository."
        exit 1
    fi

    local git_root_dir=$(git rev-parse --show-toplevel)
    local current_dir=$(pwd)

    if [[ "$git_root_dir" != "$current_dir" ]]; then
        echo "Error: This script must be run from the root of the Git repository. Current directory is not the root."
        exit 1
    fi
}

ask_for_confirmation() {
    local command=$1
    read -p "Was this the intended change? (Y/N): " answer
    if [[ $answer =~ ^[Yy]$ ]]; then
        echo "Running command without dry-run: $command"
        $command
    else
        echo "Exiting without making changes."
        exit 1
    fi
}

ask_and_tag_release() {
    local tag_name=$1
    local version=$2
    local tag_base_name=$3
    read -p "Do you want to tag this commit with: $tag_name ? (Y/N): " confirm_tag
    if [[ $confirm_tag =~ ^[Yy]$ ]]; then
        echo "Tagging the commit with tag: $tag_name"
        git commit -a -m "Bump $tag_base_name to $version"
        git tag $tag_name
        # Optionally, push the tag to remote repository
        # git push origin $tag
    else
        echo "Not tagging."
    fi
}

git_commit_new_dev_version () {
    local version=$1
    local tag_base_name=$2
    git commit -a -m "Bump $tag_base_name to next dev version $version"
}

increment_version() {
    local version=$1
    local IFS='.'  # Internal Field Separator for splitting version parts
    read -r -a parts <<< "$version"  # Read version into an array

    # Validate version format (basic check)
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: version=$version format must be X.Y.Z (e.g., 0.0.7)" >&2
        exit 1
    fi

    # Increment the patch version
    ((parts[2]++))

    # Reassemble the version and append -dev suffix
    local new_version="${parts[0]}.${parts[1]}.${parts[2]}-dev"

    echo "$new_version"
}
