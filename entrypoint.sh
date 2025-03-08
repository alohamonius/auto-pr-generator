#!/bin/bash
set -e

# Get inputs with defaults
BASE_BRANCH=${1:-main}
TEMPLATE_PATH=${2:-pr_template.hbs}
EXCLUDE_PATTERNS=${3:-"**/dist/**,**/node_modules/**,**/scripts/**,**/package-lock.json,**/lib/**"}

# Copy template to workspace if custom template exists
if [ -f "$GITHUB_WORKSPACE/$TEMPLATE_PATH" ]; then
    cp "$GITHUB_WORKSPACE/$TEMPLATE_PATH" /usr/local/bin/pr_template.hbs
fi

# Run the PR generator
cd $GITHUB_WORKSPACE
auto-pr

# If PR description was generated, set it as output
if [ -f "pr_prompt.md" ]; then
    # Escape special characters for GitHub Actions
    description=$(cat pr_prompt.md | sed 's/%/%25/g' | sed 's/\r/%0D/g' | sed 's/\n/%0A/g')
    echo "description=$description" >> $GITHUB_OUTPUT
    
    # If this is a PR event, update the PR description
    if [ "$GITHUB_EVENT_NAME" = "pull_request" ]; then
        PR_NUMBER=$(jq --raw-output .pull_request.number "$GITHUB_EVENT_PATH")
        
        if [ -n "$GITHUB_TOKEN" ]; then
            curl -X PATCH \
                -H "Authorization: token $GITHUB_TOKEN" \
                -H "Accept: application/vnd.github.v3+json" \
                "https://api.github.com/repos/$GITHUB_REPOSITORY/pulls/$PR_NUMBER" \
                -d "{\"body\": $(jq -R -s '.' < pr_prompt.md)}"
        fi
    fi
fi 