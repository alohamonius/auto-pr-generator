#!/bin/bash

current_branch=$(git rev-parse --abbrev-ref HEAD)
git_diff=$(git diff main...$current_branch -- ':!package-lock.json')
git_log=$(git log main...$current_branch)

cat > pr_template.hbs << TEMPLATE
Project Path: {{ absolute_code_path }}
I want you to generate a high-quality well-crafted Github pull request description for this project.
I will provide you with the source tree, git diff, git log, and pull request template.

Source Tree: {{ source_tree }}

Git diff:
\`\`\`
${git_diff}
\`\`\`

Git log:
\`\`\`
${git_log}
\`\`\`

The Pull Request description should follow this template:

Title: provide with concise and informative title.
# What is this?
- Explain the motivation why this is needed and the expected outcome of implementing this.
- Write it in a humanized manner.

# Changes
- Provide list of key changes with good structure.
- Mention the class name, function name, and file name.
- Explain the code changes.

For example:
# Changes
## Added Features:
   1. **New Functions in \`file_name.\`**:
       - \`function_name.\`: code description.
## Code Changes:
   1. **In \`file_name.\`**:
## Documentation Updates:
   1. **In \`file_name.\`**:

# Demo
- N/A
# Context
- N/A
TEMPLATE

code2prompt . -t pr_template.hbs --exclude="**/dist/**,**/node_modules/**,**/scripts/**,**/package-lock.json,**/lib/**" --exclude-from-tree > pr_prompt.md
