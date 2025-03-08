# Auto PR Generator

A GitHub Action that automatically generates high-quality pull request descriptions using Claude AI.

## Features

- Analyzes git diff and commit history
- Generates comprehensive, well-structured PR descriptions
- Customizable templates
- Seamless integration with GitHub workflows

## Usage

Add this action to your workflow:

```yaml
name: Generate PR Description

on:
  pull_request:
    types: [opened]

jobs:
  generate-description:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0

      - name: Generate PR Description
        uses: yourusername/auto-pr-generator@v1
        with:
          claude-api-key: ${{ secrets.CLAUDE_API_KEY }}
          github-token: ${{ secrets.TOKEN_GITHUB }}
```

## Inputs

| Input            | Description                                | Required | Default                         |
| ---------------- | ------------------------------------------ | -------- | ------------------------------- |
| `claude-api-key` | Claude API key for generating descriptions | Yes      |                                 |
| `github-token`   | GitHub token for PR access                 | Yes      |                                 |
| `template`       | PR template to use                         | No       | `default`                       |
| `exclude`        | Files/patterns to exclude                  | No       | `**/node_modules/**,**/dist/**` |

## Output Example

```
# What is this?
- This PR adds user authentication functionality to our application
- It implements the login flow according to the design specs

# Changes
## Added Features:
   1. **New Functions in `auth.js`**:
       - `authenticateUser()`: Handles user credentials validation
       - `generateToken()`: Creates JWT tokens

## Code Changes:
   1. **In `routes.js`**:
       - Added authentication middleware
       - Updated route handlers to check auth status

## Documentation Updates:
   1. **In `README.md`**:
       - Added authentication flow documentation
```

## Customizing Templates

Place your custom Handlebars template in your repository at `.github/pr-templates/custom.hbs` and specify `template: custom` in the action inputs.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

````
jobs:
  generate-pr:
    permissions:
      contents: read
      pull-requests: write
      issues: write
    runs-on: ubuntu-latest
    steps:
      - uses: your-username/auto-pr@v1
        with:
          claude-api-key: ${{ secrets.CLAUDE_API_KEY }}
          ```
````
