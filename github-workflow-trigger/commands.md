

### From the github-workflow-trigger directory ###

# to login 
wrangler login

# to deploy
wrangler deploy

# Check worker status
wrangler tail


# List all your workers
wrangler deployments list

# Delete the worker
wrangler delete

# Update secrets
wrangler secret put GITHUB_TOKEN

# List all secrets (won't show values)
wrangler secret list