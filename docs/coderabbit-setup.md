# CodeRabbit Configuration Guide

This guide explains how to configure CodeRabbit to only run after all tests pass.

## Overview

CodeRabbit is installed as a GitHub App and runs automatically on pull requests. To ensure it only provides reviews after all tests pass, we use a combination of:

1. A composite status check workflow ([`all-tests-passed.yml`](../.github/workflows/all-tests-passed.yml))
2. GitHub branch protection rules

## How It Works

### 1. Composite Status Check Workflow

The [`all-tests-passed.yml`](../.github/workflows/all-tests-passed.yml) workflow:
- Triggers on pull requests and pushes to `main` and `develop` branches
- Also triggers when other test workflows complete
- Checks the status of all required test workflows:
  - **CI**: Rust tests, formatting, and linting
  - **Code Coverage**: Test coverage analysis
  - **Security Audit**: Security vulnerability scanning
  - **Go Bindings**: Go language binding tests
  - **Python Bindings**: Python language binding tests
  - **GUI Application**: GUI build verification
- Only passes if all required workflows succeed
- Provides a single status check: "All Tests Passed"

### 2. Branch Protection Rules

By configuring branch protection to require the "All Tests Passed" check, you ensure:
- Pull requests cannot be merged until all tests pass
- CodeRabbit sees the same requirements and waits accordingly
- A single, clear status check instead of tracking multiple individual jobs

## Setup Instructions

### Step 1: Verify the Workflow

The workflow file is already created at `.github/workflows/all-tests-passed.yml`. It will automatically run on your next pull request.

### Step 2: Configure Branch Protection Rules

1. **Navigate to Repository Settings**
   - Go to your GitHub repository
   - Click on **Settings** (top navigation)
   - Click on **Branches** (left sidebar)

2. **Add or Edit Branch Protection Rule**
   - Click **Add rule** (or edit existing rule for `main` or `develop`)
   - In "Branch name pattern", enter: `main` (create separate rules for `develop` if needed)

3. **Enable Required Status Checks**
   - Check ✅ **Require status checks to pass before merging**
   - Check ✅ **Require branches to be up to date before merging** (recommended)
   
4. **Select Required Status Checks**
   - In the search box, type: `All Tests Passed`
   - Select the ✅ **All Tests Passed** check
   - You can also add individual workflow checks if desired, but the composite check is sufficient

5. **Additional Recommended Settings**
   - ✅ **Require a pull request before merging**
   - ✅ **Require approvals** (set to 1 or more)
   - ✅ **Dismiss stale pull request approvals when new commits are pushed**
   - ✅ **Require review from Code Owners** (if you have a CODEOWNERS file)
   - ✅ **Do not allow bypassing the above settings** (for stricter enforcement)

6. **Save Changes**
   - Scroll down and click **Create** or **Save changes**

### Step 3: Repeat for Other Branches

Create similar rules for:
- `develop` branch
- Any other protected branches (e.g., `release/*`, `hotfix/*`)

## How CodeRabbit Responds

Once branch protection is configured:

1. **On Pull Request Creation**:
   - All test workflows start running
   - The "All Tests Passed" workflow waits for them to complete
   - CodeRabbit sees the required status checks

2. **When Tests Are Running**:
   - The PR shows "Some checks haven't completed yet"
   - CodeRabbit may start its analysis but won't block the merge button
   - The "All Tests Passed" check shows as pending

3. **When Tests Pass**:
   - The "All Tests Passed" check turns green ✅
   - CodeRabbit completes its review
   - The PR becomes mergeable (if all other requirements are met)

4. **When Tests Fail**:
   - The "All Tests Passed" check fails ❌
   - The PR cannot be merged
   - CodeRabbit may still provide its review, but merge is blocked

## Troubleshooting

### The "All Tests Passed" check doesn't appear

- Make sure you've pushed the workflow file to your repository
- Create a new pull request to trigger the workflow
- Check the Actions tab to see if the workflow is running

### Some workflows are not being checked

- Verify the workflow names in `all-tests-passed.yml` match your actual workflow files
- Check that the workflows are triggered for the same events (pull_request, push)
- Some workflows may be skipped if they don't apply to certain branches

### CodeRabbit still runs before tests complete

- Verify branch protection rules are properly configured
- Check that "All Tests Passed" is selected as a required status check
- CodeRabbit may start analysis early but won't affect merge requirements

### Tests pass but "All Tests Passed" fails

- Check the workflow run logs in the Actions tab
- The workflow may be having trouble querying the GitHub API
- Ensure the `GITHUB_TOKEN` has sufficient permissions

## Customization

### Adding More Required Workflows

Edit `.github/workflows/all-tests-passed.yml` and add workflow names to the `workflows` array:

```yaml
const workflows = [
  'CI',
  'Code Coverage',
  'Security Audit',
  'Go Bindings',
  'Python Bindings',
  'GUI Application',
  'Your New Workflow Name'  # Add here
];
```

### Changing Which Branches Are Protected

Edit the `on` section in `all-tests-passed.yml`:

```yaml
on:
  pull_request:
    branches: [ main, develop, release/* ]  # Add more branches
  push:
    branches: [ main, develop, release/* ]
```

### Making Certain Checks Optional

In the workflow script, you can modify the logic to allow certain workflows to be skipped:

```javascript
if (status === 'success' || status === 'skipped') {
  // Treat skipped as success for optional workflows
}
```

## Benefits

✅ **Single Status Check**: One clear indicator instead of tracking multiple jobs  
✅ **Consistent Enforcement**: Same rules apply to all contributors  
✅ **Better PR Experience**: Clear indication of what's blocking merge  
✅ **CodeRabbit Integration**: Reviews happen in context of passing tests  
✅ **Flexible**: Easy to add or remove required workflows  

## Related Documentation

- [GitHub Branch Protection Rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [GitHub Actions Status Checks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/about-status-checks)
- [CodeRabbit Documentation](https://docs.coderabbit.ai/)