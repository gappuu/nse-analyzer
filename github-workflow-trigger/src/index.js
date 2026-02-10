export default {
  async scheduled(event, env, ctx) {
    try {
      const response = await fetch(
        `https://api.github.com/repos/${env.OWNER}/${env.REPO}/actions/workflows/${env.WORKFLOW}/dispatches`,
        {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${env.GITHUB_TOKEN}`,
            'Accept': 'application/vnd.github.v3+json',
            'User-Agent': 'Cloudflare-Worker'
          },
          body: JSON.stringify({ ref: 'master' })
        }
      );

      if (response.ok) {
        console.log('✅ Workflow triggered successfully');
      } else {
        const error = await response.text();
        console.error('❌ Failed to trigger workflow:', error);
      }
    } catch (error) {
      console.error('❌ Error:', error);
    }
  },

  // Optional: HTTP endpoint to manually trigger
  async fetch(request, env, ctx) {
    if (request.method === 'GET') {
      return new Response('Workflow trigger is active. Use POST to trigger manually.');
    }

    // Manually trigger via POST request
    const triggerEvent = { scheduledTime: Date.now() };
    await this.scheduled(triggerEvent, env, ctx);
    
    return new Response('Workflow trigger initiated', { status: 200 });
  }
};